//! Interfaz de observación; las consultas se ejecutan fuera del hilo de entrada.
use crate::modules::{docker, health, projects, services};
use crossterm::{event::{self, Event, KeyCode, KeyEventKind}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}};
use ratatui::{backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, List, ListItem, Paragraph}, Terminal};
use std::{ffi::OsString, io::{self, stdout}, sync::mpsc::{self, Receiver}, thread, time::{Duration, Instant}};

const NAMES: [&str; 4] = ["Health", "Services", "Docker", "Projects"];
const LIMIT: Duration = Duration::from_secs(10);

type Answer = Result<String, String>;
struct Worker { view: usize, started: Instant, receiver: Receiver<Answer>, timed_out: bool }
struct App { selected: usize, view: usize, content: [String; 4], worker: Option<Worker>, row: usize, column: usize }
impl App {
    fn new() -> Self {
        Self { selected: 0, view: 0, content: std::array::from_fn(|_| "Sin consultar. Seleccioná la vista con Enter y pulsá R para consultar.".into()), worker: None, row: 0, column: 0 }
    }
    // Los límites dependen del contenido actual, no del tamaño anterior de la ventana.
    fn clamp_scroll(&mut self) {
        let text = &self.content[self.view];
        self.row = self.row.min(text.lines().count().saturating_sub(1));
        self.column = self.column.min(text.lines().map(|line| line.chars().count()).max().unwrap_or(0).saturating_sub(1));
    }
    fn visible_content(&self, height: usize) -> String {
        self.content[self.view].lines().skip(self.row).take(height)
            .map(|line| line.chars().skip(self.column).collect::<String>())
            .collect::<Vec<_>>().join("\n")
    }
    fn request(&mut self) {
        if self.worker.is_some() {
            self.content[self.view] = "Hay una consulta anterior en curso; esperá a que termine. No se crean más procesos de consulta.".into();
            return;
        }
        let view = self.view;
        let (sender, receiver) = mpsc::channel();
        match thread::Builder::new().name("javipiv2-query".into()).spawn(move || { let _ = sender.send(query(view)); }) {
            Ok(_) => {
                self.content[view] = "Consultando…".into();
                self.worker = Some(Worker { view, started: Instant::now(), receiver, timed_out: false });
            }
            Err(error) => self.content[view] = format!("Error al iniciar consulta: {error}"),
        }
    }
    fn update(&mut self) {
        let Some(worker) = self.worker.as_mut() else { return };
        match worker.receiver.try_recv() {
            Ok(answer) => {
                let view = worker.view;
                self.content[view] = match answer { Ok(text) => text, Err(error) => format!("Error de lectura: {error}") };
                self.worker = None;
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.content[worker.view] = "Error: la consulta terminó inesperadamente".into();
                self.worker = None;
            }
            Err(mpsc::TryRecvError::Empty) if !worker.timed_out && worker.started.elapsed() >= LIMIT => {
                worker.timed_out = true;
                self.content[worker.view] = "Tiempo de espera agotado; la consulta sigue en segundo plano. No se permiten nuevas consultas hasta que termine.".into();
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
    fn key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Char('q' | 'Q') | KeyCode::Esc => return false,
            KeyCode::Up => self.selected = (self.selected + NAMES.len() - 1) % NAMES.len(),
            KeyCode::Down => self.selected = (self.selected + 1) % NAMES.len(),
            KeyCode::Enter => {
                if self.view != self.selected { self.view = self.selected; self.row = 0; self.column = 0; }
            }
            KeyCode::Char('j') => self.row = self.row.saturating_add(1),
            KeyCode::Char('k') => self.row = self.row.saturating_sub(1),
            KeyCode::Char('l') => self.column = self.column.saturating_add(1),
            KeyCode::Char('h') => self.column = self.column.saturating_sub(1),
            KeyCode::Char('r' | 'R') => self.request(),
            _ => {}
        }
        self.clamp_scroll();
        true
    }
}

fn query(view: usize) -> Answer {
    match view {
        0 => {
            let status = health::read_status().map_err(|e| e.to_string())?;
            let cpu = health::read_cpu_observation();
            let load = cpu.load_average.map(|v| v.join(", ")).unwrap_or_else(|| "desconocida".into());
            let temperature = cpu.cpu_temperature_millicelsius.map(|v| format!("{:.3} °C", v as f64 / 1000.0)).unwrap_or_else(|| "desconocida".into());
            Ok(format!("Host: {}\nUptime: {} h {} min\nRAM disponible: {} / {} MiB\nCarga (1, 5, 15 min): {}\nTemperatura: {}", status.hostname, status.uptime_seconds / 3600, status.uptime_seconds / 60 % 60, status.memory_available_kib / 1024, status.memory_total_kib / 1024, load, temperature))
        }
        1 => {
            let entries = services::list()?;
            let mut text = String::from("Servicios conocidos por systemd (no inventario completo)\nNombre | Actividad | Subestado\n");
            for entry in entries { text.push_str(&format!("{} | {} | {}\n", entry.name, entry.active.as_deref().unwrap_or("desconocido"), entry.sub.as_deref().unwrap_or("desconocido"))); }
            text.push_str("Controles de servicios: pendientes.");
            Ok(text)
        }
        2 => {
            let entries = docker::list()?;
            let mut text = String::from("Nombre | Imagen | Estado\n");
            if entries.is_empty() { text.push_str("No hay contenedores.\n"); }
            for entry in entries { text.push_str(&format!("{} | {} | {}\n", entry.name, entry.image, entry.state)); }
            text.push_str("Controles Docker: pendientes.");
            Ok(text)
        }
        3 => {
            let workspace = std::env::var_os("JAVIPIV2_WORKSPACES").unwrap_or_else(|| OsString::from("/srv/workspaces"));
            let entries = projects::Projects::new(workspace)?.list()?;
            let mut text = String::from("Nombre | Git | Rama | HEAD\n");
            if entries.is_empty() { text.push_str("No hay proyectos.\n"); }
            for entry in entries {
                let git = match entry.git { projects::GitState::Compatible => "compatible (validez no verificada)", projects::GitState::NotGit => "sin Git", projects::GitState::Unknown => "desconocido" };
                text.push_str(&format!("{} | {} | {} | {}\n", entry.name, git, entry.branch.as_deref().unwrap_or("desconocido"), entry.head.as_deref().unwrap_or("desconocido")));
            }
            Ok(text)
        }
        _ => Err("vista desconocida".into()),
    }
}

// La limpieza intenta ambas operaciones incluso si una falla, también ante setup parcial.
struct Screen { raw: bool, alternate: bool }
impl Screen {
    fn enter() -> io::Result<Self> {
        let mut screen = Self { raw: false, alternate: false };
        enable_raw_mode()?;
        screen.raw = true;
        // Una escritura parcial también requiere intentar restaurar la pantalla.
        screen.alternate = true;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(screen)
    }
    fn restore(&mut self) -> io::Result<()> {
        let alternate = if self.alternate { execute!(stdout(), LeaveAlternateScreen) } else { Ok(()) };
        let raw = if self.raw { disable_raw_mode() } else { Ok(()) };
        if alternate.is_ok() { self.alternate = false; }
        if raw.is_ok() { self.raw = false; }
        alternate.and(raw)
    }
}
impl Drop for Screen { fn drop(&mut self) { let _ = self.restore(); } }

pub fn run() -> Result<(), String> {
    let mut screen = Screen::enter().map_err(|e| format!("terminal: {e}"))?;
    let result = (|| -> io::Result<()> {
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        let mut app = App::new();
        app.request();
        loop {
            app.update();
            app.clamp_scroll();
            terminal.draw(|frame| {
                let areas = Layout::default().direction(Direction::Horizontal).constraints([Constraint::Length(20), Constraint::Min(1)]).split(frame.area());
                let items = NAMES.iter().enumerate().map(|(i, name)| ListItem::new(format!("{} {}", if i == app.selected { ">" } else { " " }, name))).collect::<Vec<_>>();
                frame.render_widget(List::new(items).block(Block::default().borders(Borders::ALL).title("Módulos")), areas[0]);
                let text = app.visible_content(usize::from(areas[1].height.saturating_sub(2)));
                frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL)
                    .title(format!("{} — ↑↓ menú Enter abrir R consultar Q/Esc salir", NAMES[app.view]))
                    .title_bottom("j/k vertical · h/l horizontal")), areas[1]);
            })?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && !app.key(key.code) { break; }
                }
            }
        }
        Ok(())
    })();
    let cleanup = screen.restore();
    result.and(cleanup).map_err(|e| format!("terminal: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_separates_selection_from_opening() {
        let mut app = App::new();
        assert!(app.key(KeyCode::Down));
        assert_eq!((app.selected, app.view), (1, 0));
        assert!(app.key(KeyCode::Enter));
        assert_eq!(app.view, 1);
        assert!(app.worker.is_none());
        assert!(!app.key(KeyCode::Esc));
    }
    #[test]
    fn scroll_reaches_long_rows_and_is_bounded() {
        let mut app = App::new();
        app.content[0] = "primera\nsegunda fila larga\núltima".into();
        for _ in 0..20 { app.key(KeyCode::Char('j')); }
        assert_eq!(app.row, 2);
        assert_eq!(app.visible_content(1), "última");
        app.key(KeyCode::Char('k'));
        for _ in 0..7 { app.key(KeyCode::Char('l')); }
        assert_eq!(app.visible_content(1), "fila larga");
        for _ in 0..100 { app.key(KeyCode::Char('l')); }
        assert_eq!(app.column, "segunda fila larga".chars().count() - 1);
        for _ in 0..100 { app.key(KeyCode::Char('h')); }
        for _ in 0..100 { app.key(KeyCode::Char('k')); }
        assert_eq!((app.row, app.column), (0, 0));
    }
    #[test]
    fn switching_and_content_changes_bound_scroll() {
        let mut app = App::new();
        app.content[0] = "abc\ndef".into();
        app.key(KeyCode::Char('j'));
        app.key(KeyCode::Char('l'));
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!((app.row, app.column), (0, 0));
        app.row = 100;
        app.column = 100;
        app.content[1] = "x".into();
        app.clamp_scroll();
        assert_eq!((app.row, app.column), (0, 0));
    }
    #[test]
    fn docker_and_projects_require_explicit_request() {
        let mut app = App::new();
        for _ in 0..2 { app.key(KeyCode::Down); }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 2);
        assert!(app.worker.is_none());
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 3);
        assert!(app.worker.is_none());
    }
}
