//! Interfaz de observación; las consultas se ejecutan fuera del hilo de entrada.
use crate::modules::{
    agents, clean, developer, docker, health, projects, remote, services, storage, updates,
    watchdog,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use std::{
    ffi::OsString,
    io::{self, stdout},
    sync::mpsc::{self, Receiver},
    thread,
    time::{Duration, Instant},
};

const NAMES: [&str; 12] = [
    "Health",
    "Services",
    "Docker",
    "Projects",
    "Storage",
    "Updates",
    "Agents",
    "Remote",
    "Watchdog",
    "Developer",
    "Overview",
    "Clean",
];
const LIMIT: Duration = Duration::from_secs(10);

#[derive(Debug)]
enum Data {
    Health(String, bool),
    Clean(Option<storage::Capacity>),
    Watchdog(watchdog::Assessment),
    Services(Vec<services::Service>),
    Docker(Vec<docker::Container>),
    Projects(Vec<projects::Project>),
    Storage(storage::StorageStatus, Option<storage::Capacity>),
    Updates(Vec<updates::Observation>),
    Agents(Vec<agents::Observation>),
    Remote(Vec<remote::Observation>),
    Developer(Vec<agents::Observation>),
}
impl Data {
    fn has_evidence(&self) -> bool {
        let known = |value: &Option<String>| {
            value
                .as_ref()
                .is_some_and(|s| !s.is_empty() && s != "-" && s != "desconocido")
        };
        match self {
            Self::Health(_, succeeded) => *succeeded,
            Self::Clean(capacity) => capacity.is_some(),
            Self::Remote(v) => v.iter().any(remote::Observation::has_evidence),
            Self::Watchdog(a) => *a != watchdog::Assessment::Unknown,
            Self::Services(v) => v
                .iter()
                .any(|s| known(&s.load) || known(&s.active) || known(&s.sub)),
            Self::Docker(v) => !v.is_empty(),
            Self::Projects(v) => !v.is_empty(),
            Self::Storage(s, c) => {
                s.autofs || s.srv_ext4 || !s.docker_overlays.is_empty() || c.is_some()
            }
            Self::Updates(v) => v.iter().any(|o| known(&o.installed) || known(&o.upstream)),
            Self::Agents(v) | Self::Developer(v) => {
                v.iter().any(|o| known(&o.version) || known(&o.executable))
            }
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Health(_, _) | Self::Watchdog(_) | Self::Clean(_) => 0,
            Self::Storage(_, _) => 1,
            Self::Services(v) => v.len(),
            Self::Docker(v) => v.len(),
            Self::Projects(v) => v.len(),
            Self::Updates(v) => v.len(),
            Self::Agents(v) => v.len(),
            Self::Remote(v) => v.len(),
            Self::Developer(v) => v.len(),
        }
    }
    fn line(&self, index: usize, detail: bool) -> Option<String> {
        let unknown = |v: &Option<String>| {
            v.as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("desconocido")
                .to_owned()
        };
        match self {
            Self::Health(_, _) | Self::Watchdog(_) | Self::Clean(_) => None,
            Self::Updates(v) => v.get(index).map(updates::Observation::line),
            Self::Agents(v) => v.get(index).map(agents::Observation::line),
            Self::Remote(v) => v.get(index).map(remote::Observation::line),
            Self::Developer(v) => v.get(index).map(|item| developer::status_from(vec![item.clone()]).remove(0)),
            Self::Storage(status, capacity) => (index == 0).then(|| {
                let capacity = capacity.map_or_else(|| "desconocido".to_owned(), |c| format!(
                    "total={} B, usada={} B, disponible={} B, reservada={} B", c.total_bytes, c.used_bytes, c.available_bytes, c.reserved_bytes));
                format!(
                    "Autofs en /srv: {}\nMontaje ext4 activo: {}\nPunto de montaje ext4: {}\nOverlays Docker reconocidos: {}\nCapacidad /srv: {}",
                    if status.autofs { "identificado" } else { "desconocido" },
                    if status.srv_ext4 { "identificado" } else { "desconocido" },
                    if status.srv_ext4 { "/srv" } else { "desconocido" },
                    status.docker_overlays.len(), capacity
                )
            }),
            Self::Services(v) => v.get(index).map(|e| {
                if detail {
                    format!(
                        "Nombre: {}\nCarga: {}\nActividad: {}\nSubestado: {}\nCategoría: {}",
                        e.name,
                        unknown(&e.load),
                        unknown(&e.active),
                        unknown(&e.sub),
                        e.category()
                    )
                } else {
                    format!("{} | {} | {}", e.name, unknown(&e.active), unknown(&e.sub))
                }
            }),
            Self::Docker(v) => v.get(index).map(|e| {
                if detail {
                    format!(
                        "Nombre: {}\nIdentificador: {}\nImagen: {}\nEstado: {}",
                        e.name, e.id, e.image, e.state
                    )
                } else {
                    format!("{} | {} | {}", e.name, e.image, e.state)
                }
            }),
            Self::Projects(v) => v.get(index).map(|e| {
                let git = match e.git {
                    projects::GitState::Compatible => "compatible (validez no verificada)",
                    projects::GitState::NotGit => "sin Git",
                    projects::GitState::Unknown => "desconocido",
                };
                if detail {
                    format!(
                        "Nombre: {}\nEvidencia Git: {}\nRama: {}\nHEAD: {}",
                        e.name,
                        git,
                        unknown(&e.branch),
                        unknown(&e.head)
                    )
                } else {
                    format!(
                        "{} | {} | {} | {}",
                        e.name,
                        git,
                        unknown(&e.branch),
                        unknown(&e.head)
                    )
                }
            }),
        }
    }
}
type Answer = Result<Data, String>;
fn query_error(view: usize) -> &'static str {
    match view {
        0 => "Error al consultar Health; datos anteriores",
        1 => "Error al consultar servicios (systemd); datos anteriores",
        2 => "Error al consultar Docker; datos anteriores",
        3 => "Error al consultar Projects; datos anteriores",
        4 => "Error al consultar Storage (/proc/self/mountinfo); datos anteriores",
        5 => "Error al consultar Updates; datos anteriores",
        6 => "Error al consultar Agents; datos anteriores",
        7 => "Error al consultar Remote; datos anteriores",
        8 => "Error al consultar Watchdog; datos anteriores",
        9 => "Error al consultar Developer; datos anteriores",
        11 => "Error al consultar Clean; datos anteriores",
        _ => "Error al consultar vista; datos anteriores",
    }
}
struct Worker {
    view: usize,
    started: Instant,
    receiver: Receiver<Answer>,
    timed_out: bool,
}
struct App {
    selected: usize,
    view: usize,
    content: [String; 12],
    data: [Option<Data>; 12],
    status: [Option<String>; 12],
    failed: [bool; 12],
    last_error: [Option<String>; 12],
    worker: Option<Worker>,
    notice: Option<String>,
    row: usize,
    column: usize,
    list_row: usize,
    list_column: usize,
    item: [usize; 12],
    table_focus: bool,
    detail: bool,
    height: usize,
}
impl App {
    fn new() -> Self {
        Self {
            selected: 0,
            view: 0,
            content: std::array::from_fn(|_| {
                "Sin consultar. Seleccioná la vista con Enter y pulsá R para consultar.".into()
            }),
            data: std::array::from_fn(|_| None),
            status: std::array::from_fn(|_| None),
            failed: [false; 12],
            last_error: std::array::from_fn(|_| None),
            worker: None,
            notice: None,
            row: 0,
            column: 0,
            list_row: 0,
            list_column: 0,
            item: [0; 12],
            table_focus: false,
            detail: false,
            height: 0,
        }
    }
    fn text(&self) -> String {
        if self.view == 10 {
            let mut text = String::from("Solo resultados almacenados; disponible indica evidencia, no instalación ni salud. Remote no demuestra conectividad. R no consulta.\n");
            for (i, name) in NAMES.iter().enumerate().filter(|(i, _)| *i != 10) {
                let state = if self.failed[i] {
                    "error"
                } else {
                    match &self.data[i] {
                        None => "no consultado",
                        Some(data) if data.has_evidence() => "disponible",
                        Some(_) => "desconocido",
                    }
                };
                text.push_str(&format!("\n{name}: {state}"));
                if self.failed[i] {
                    if self.data[i].is_some() {
                        text.push_str(" (datos anteriores, pueden estar desactualizados)");
                    }
                    if let Some(status) = &self.last_error[i] {
                        text.push_str(&format!(" — {status}"));
                    }
                }
            }
            return text;
        }
        let mut text = match &self.data[self.view] {
            None => self.content[self.view].clone(),
            Some(Data::Health(text, _)) => text.clone(),
            Some(Data::Clean(capacity)) => clean::format_observation(*capacity),
            Some(Data::Watchdog(assessment)) => assessment.line(),
            Some(data) if self.detail => data
                .line(self.item[self.view], true)
                .unwrap_or_else(|| "Sin elementos.".into()),
            Some(data) if data.len() == 0 => "Sin elementos.".into(),
            Some(data) => (0..data.len())
                .filter_map(|i| {
                    data.line(i, false).map(|line| {
                        format!(
                            "{} {}",
                            if i == self.item[self.view] { ">" } else { " " },
                            line
                        )
                    })
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        if self.view == 8 {
            text = format!("Umbral ilustrativo fijo: 20% de memoria disponible; no diagnóstico. Solo evalúa memoria, no la salud general. Sin supervisión ni acciones automáticas.\n\n{text}");
        }
        if self.view == 7 {
            text.push_str("\n\nEstado de servicios locales; no demuestra conectividad.");
        }
        if self.view == 1 && !self.detail {
            text.push_str("\n\nServicios conocidos por systemd; no es un inventario completo.");
        }
        if let Some(status) = &self.status[self.view] {
            text.push_str("\n\nEstado de consulta (j/k y h/l para leer):\n");
            text.push_str(status);
        }
        if let Some(notice) = &self.notice {
            text.push_str("\n\nAviso:\n");
            text.push_str(notice);
        }
        text
    }
    fn clamp_scroll(&mut self) {
        let text = self.text();
        self.row = self.row.min(text.lines().count().saturating_sub(1));
        let table = !self.detail
            && self.data[self.view]
                .as_ref()
                .is_some_and(|d| !matches!(d, Data::Health(_, _)) && d.len() > 0);
        let rows = self.data[self.view].as_ref().map_or(0, Data::len);
        let max_column = text
            .lines()
            .enumerate()
            .map(|(index, line)| {
                line.chars()
                    .count()
                    .saturating_sub(if table && index < rows { 3 } else { 1 })
            })
            .max()
            .unwrap_or(0);
        self.column = self.column.min(max_column);
    }
    fn keep_visible(&mut self) {
        if self.height == 0 {
            return;
        }
        let item = self.item[self.view];
        if item < self.row {
            self.row = item;
        }
        if item >= self.row.saturating_add(self.height) {
            self.row = item + 1 - self.height;
        }
    }
    fn visible_content(&self, height: usize) -> String {
        let table = !self.detail
            && self.data[self.view]
                .as_ref()
                .is_some_and(|d| !matches!(d, Data::Health(_, _)) && d.len() > 0);
        self.text()
            .lines()
            .enumerate()
            .skip(self.row)
            .take(height)
            .map(|(index, line)| {
                if table && index < self.data[self.view].as_ref().unwrap().len() {
                    let mut chars = line.chars();
                    let marker = chars.next().unwrap_or(' ');
                    chars.next();
                    format!("{} {}", marker, chars.skip(self.column).collect::<String>())
                } else {
                    line.chars().skip(self.column).collect::<String>()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn request(&mut self) {
        self.request_with(query);
    }
    fn request_with(&mut self, query_fn: fn(usize) -> Answer) {
        if self.view == 10 {
            return;
        }
        if self.worker.is_some() {
            self.notice = Some("Hay una consulta en curso; esperá a que termine".into());
            return;
        }
        self.notice = None;
        let view = self.view;
        let (sender, receiver) = mpsc::channel();
        match thread::Builder::new()
            .name("javipiv2-query".into())
            .spawn(move || {
                let _ = sender.send(query_fn(view));
            }) {
            Ok(_) => {
                self.status[view] = Some("Consultando… (datos anteriores)".into());
                self.worker = Some(Worker {
                    view,
                    started: Instant::now(),
                    receiver,
                    timed_out: false,
                });
            }
            Err(_) => {
                self.failed[view] = true;
                self.status[view] = Some("Error al iniciar consulta; datos anteriores".into());
                self.last_error[view] = self.status[view].clone();
            }
        }
    }
    fn update(&mut self) {
        let Some(worker) = self.worker.as_mut() else {
            return;
        };
        match worker.receiver.try_recv() {
            Ok(answer) => {
                self.notice = None;
                let view = worker.view;
                match answer {
                    Ok(data) => {
                        self.item[view] = self.item[view].min(data.len().saturating_sub(1));
                        self.data[view] = Some(data);
                        self.status[view] = None;
                        self.failed[view] = false;
                        self.last_error[view] = None;
                        if view == self.view && self.detail {
                            self.detail = false;
                            self.row = self.list_row;
                            self.column = self.list_column;
                        }
                    }
                    Err(_) => {
                        self.failed[view] = true;
                        self.status[view] = Some(query_error(view).into());
                        self.last_error[view] = self.status[view].clone();
                    }
                }
                self.worker = None;
                if view == self.view && !self.detail {
                    self.keep_visible();
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.failed[worker.view] = true;
                self.status[worker.view] =
                    Some("Error: la consulta terminó inesperadamente (datos anteriores)".into());
                self.last_error[worker.view] = self.status[worker.view].clone();
                self.notice = None;
                self.worker = None;
            }
            Err(mpsc::TryRecvError::Empty)
                if !worker.timed_out && worker.started.elapsed() >= LIMIT =>
            {
                worker.timed_out = true;
                self.status[worker.view] = Some("Datos anteriores · consulta en curso por más de diez segundos, no cancelada. No se permiten nuevas consultas hasta que termine.".into());
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }
    }
    fn key(&mut self, key: KeyCode) -> bool {
        self.key_with(key, query)
    }
    fn key_with(&mut self, key: KeyCode, query_fn: fn(usize) -> Answer) -> bool {
        match key {
            KeyCode::Char('q' | 'Q') => return false,
            KeyCode::Esc if self.detail => {
                self.detail = false;
                self.row = self.list_row;
                self.column = self.list_column;
                self.keep_visible();
            }
            KeyCode::Esc if self.table_focus => self.table_focus = false,
            KeyCode::Esc => return false,
            KeyCode::Tab if !self.detail && self.view != 0 && self.view != 10 => {
                self.table_focus = !self.table_focus;
                if self.table_focus {
                    self.keep_visible();
                }
            }
            KeyCode::Up | KeyCode::Down if self.table_focus && !self.detail => {
                let len = self.data[self.view].as_ref().map_or(0, Data::len);
                if len > 0 {
                    self.item[self.view] = if key == KeyCode::Up {
                        self.item[self.view].saturating_sub(1)
                    } else {
                        (self.item[self.view] + 1).min(len - 1)
                    };
                    self.keep_visible();
                }
            }
            KeyCode::Up if !self.detail => {
                self.selected = (self.selected + NAMES.len() - 1) % NAMES.len()
            }
            KeyCode::Down if !self.detail => self.selected = (self.selected + 1) % NAMES.len(),
            KeyCode::Enter if self.table_focus && !self.detail => {
                if self.data[self.view].as_ref().is_some_and(|d| d.len() > 0) {
                    self.detail = true;
                    self.list_row = self.row;
                    self.list_column = self.column;
                    self.row = 0;
                    self.column = 0;
                }
            }
            KeyCode::Enter if !self.detail => {
                if self.view != self.selected {
                    self.view = self.selected;
                    self.table_focus = false;
                    self.row = 0;
                    self.column = 0;
                    self.notice = None;
                    if self.view != 0 {
                        self.keep_visible();
                    }
                }
            }
            KeyCode::Char('j') => self.row = self.row.saturating_add(1),
            KeyCode::Char('k') => self.row = self.row.saturating_sub(1),
            KeyCode::Char('l') => self.column = self.column.saturating_add(1),
            KeyCode::Char('h') => self.column = self.column.saturating_sub(1),
            KeyCode::Char('r' | 'R') => self.request_with(query_fn),
            _ => {}
        }
        self.clamp_scroll();
        if matches!(key, KeyCode::Char('j' | 'k'))
            && self.table_focus
            && !self.detail
            && self.data[self.view].as_ref().is_some_and(|d| d.len() > 0)
        {
            self.keep_visible();
        }
        true
    }
}

fn query(view: usize) -> Answer {
    match view {
        0 => {
            let status = health::read_status().map_err(|e| e.to_string())?;
            let cpu = health::read_cpu_observation();
            let load = cpu
                .load_average
                .map(|v| v.join(", "))
                .unwrap_or_else(|| "desconocida".into());
            let temperature = cpu
                .cpu_temperature_millicelsius
                .map(|v| format!("{:.3} °C", v as f64 / 1000.0))
                .unwrap_or_else(|| "desconocida".into());
            Ok(Data::Health(format!("Host: {}\nUptime: {} h {} min\nRAM disponible: {} / {} MiB\nCarga (1, 5, 15 min): {}\nTemperatura: {}", status.hostname, status.uptime_seconds / 3600, status.uptime_seconds / 60 % 60, status.memory_available_kib / 1024, status.memory_total_kib / 1024, load, temperature), true))
        }
        1 => services::list().map(Data::Services),
        2 => docker::list().map(Data::Docker),
        3 => {
            let workspace = std::env::var_os("JAVIPIV2_WORKSPACES")
                .unwrap_or_else(|| OsString::from("/srv/workspaces"));
            projects::Projects::new(workspace)?
                .list()
                .map(Data::Projects)
        }
        4 => storage::read_capacity().map(|(status, capacity)| Data::Storage(status, capacity)),
        5 => Ok(Data::Updates(updates::check())),
        6 => Ok(Data::Agents(agents::status())),
        7 => Ok(Data::Remote(remote::status())),
        8 => Ok(Data::Watchdog(watchdog::status("20"))),
        9 => Ok(Data::Developer(agents::status())),
        11 => Ok(Data::Clean(clean::observe())),
        _ => Err("vista desconocida".into()),
    }
}

// La limpieza intenta ambas operaciones incluso si una falla, también ante setup parcial.
struct Screen {
    raw: bool,
    alternate: bool,
}
impl Screen {
    fn enter() -> io::Result<Self> {
        let mut screen = Self {
            raw: false,
            alternate: false,
        };
        enable_raw_mode()?;
        screen.raw = true;
        screen.alternate = true;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(screen)
    }
    fn restore(&mut self) -> io::Result<()> {
        let alternate = if self.alternate {
            execute!(stdout(), LeaveAlternateScreen)
        } else {
            Ok(())
        };
        let raw = if self.raw { disable_raw_mode() } else { Ok(()) };
        if alternate.is_ok() {
            self.alternate = false;
        }
        if raw.is_ok() {
            self.raw = false;
        }
        alternate.and(raw)
    }
}
impl Drop for Screen {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn render_menu(frame: &mut ratatui::Frame<'_>, area: ratatui::layout::Rect, selected: usize) {
    let items = NAMES
        .iter()
        .enumerate()
        .map(|(i, name)| {
            ListItem::new(format!(
                "{} {}",
                if i == selected { ">" } else { " " },
                name
            ))
        })
        .collect::<Vec<_>>();
    let mut state = ListState::default().with_selected(Some(selected));
    frame.render_stateful_widget(
        List::new(items).block(Block::default().borders(Borders::ALL).title("Módulos")),
        area,
        &mut state,
    );
}

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
                render_menu(frame, areas[0], app.selected);
                let height = usize::from(areas[1].height.saturating_sub(2));
                if app.height != height { app.height = height; if app.view != 0 && !app.detail { app.keep_visible(); } }
                let text = app.visible_content(app.height);
                let footer = match (&app.status[app.view], &app.notice) {
                    (Some(_), Some(notice)) => format!("Estado: ver contenido · {notice}"),
                    (Some(_), None) => "Estado: ver contenido (j/k h/l)".into(),
                    (None, Some(notice)) => notice.clone(),
                    (None, None) => "Tab foco · ↑↓ seleccionar · Enter abrir · Esc volver/salir · R consultar · Q salir · j/k h/l desplazar".into(),
                };
                frame.render_widget(Paragraph::new(text).block(Block::default().borders(Borders::ALL)
                    .title(format!("{}{} — {}", NAMES[app.view], if app.view == 1 { " (systemd: no inventario completo)" } else { "" }, if app.detail { "detalle" } else if app.table_focus { "tabla" } else { "menú" }))
                    .title_bottom(footer)), areas[1]);
            })?;
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press && !app.key(key.code) {
                        break;
                    }
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
    fn clean_view_requires_manual_request() {
        fn fake(view: usize) -> Answer {
            assert_eq!(view, 11);
            Ok(Data::Clean(None))
        }
        let mut app = App::new();
        app.selected = 11;
        app.key_with(KeyCode::Enter, fake);
        assert_eq!(app.view, 11);
        assert!(app.worker.is_none());
        app.key_with(KeyCode::Char('R'), fake);
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app.text().contains("Raíz (/): desconocido"));
        app.selected = 10;
        app.key_with(KeyCode::Enter, fake);
        assert!(app.text().contains("Clean: desconocido"));
        app.data[11] = Some(Data::Clean(Some(storage::Capacity {
            total_bytes: 100,
            used_bytes: 60,
            available_bytes: 30,
            reserved_bytes: 10,
        })));
        assert!(app.text().contains("Clean: disponible"));
        app.key_with(KeyCode::Char('R'), fake);
        assert!(app.worker.is_none());
    }
    #[test]
    fn menu_selection_stays_visible_in_small_terminals() {
        use ratatui::backend::TestBackend;
        let mut app = App::new();
        for height in [3, 4, 6, 14] {
            let mut terminal = Terminal::new(TestBackend::new(20, height)).unwrap();
            let check = |terminal: &mut Terminal<TestBackend>, app: &App, expected: &str| {
                terminal
                    .draw(|frame| render_menu(frame, frame.area(), app.selected))
                    .unwrap();
                let buffer = terminal.backend().buffer();
                let rows = (0..height)
                    .map(|y| (0..20).map(|x| buffer[(x, y)].symbol()).collect::<String>())
                    .collect::<Vec<_>>();
                assert!(
                    rows.iter().any(|row| row.contains(expected)),
                    "height={height}, rows={rows:?}"
                );
                assert!(rows[0].contains("Módulos"));
                assert!(app.worker.is_none());
            };
            app.selected = 0;
            check(&mut terminal, &app, "> Health");
            app.key(KeyCode::Up);
            check(&mut terminal, &app, "> Clean");
            app.key(KeyCode::Up);
            check(&mut terminal, &app, "> Overview");
            app.key(KeyCode::Up);
            check(&mut terminal, &app, "> Developer");
            app.key(KeyCode::Down);
            app.key(KeyCode::Down);
            app.key(KeyCode::Down);
            check(&mut terminal, &app, "> Health");
            app.key(KeyCode::Down);
            check(&mut terminal, &app, "> Services");
            app.key(KeyCode::Enter);
            assert_eq!(app.view, 1);
            assert!(app.worker.is_none());
        }
        for height in [0, 1, 2] {
            let mut terminal = Terminal::new(TestBackend::new(20, height)).unwrap();
            app.selected = 10;
            terminal
                .draw(|frame| render_menu(frame, frame.area(), app.selected))
                .unwrap();
            assert!(app.worker.is_none());
        }
    }

    #[test]
    fn overview_is_passive_and_reports_cached_states() {
        fn forbidden(_: usize) -> Answer {
            panic!("overview queried")
        }
        let mut app = App::new();
        app.selected = 10;
        app.key_with(KeyCode::Enter, forbidden);
        app.key_with(KeyCode::Char('R'), forbidden);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Watchdog: no consultado"));
        app.data[8] = Some(Data::Watchdog(watchdog::Assessment::Unknown));
        assert!(app.text().contains("Watchdog: desconocido"));
        app.data[8] = Some(Data::Watchdog(watchdog::Assessment::Healthy));
        assert!(app.text().contains("Watchdog: disponible"));
        app.status[8] = Some(query_error(8).into());
        app.failed[8] = true;
        app.last_error[8] = app.status[8].clone();
        assert!(app.text().contains("Watchdog: error"));
        assert!(app.text().contains("datos anteriores"));
    }

    #[test]
    fn overview_typed_evidence_and_failed_refresh() {
        let mut app = App::new();
        app.view = 10;
        app.data[6] = Some(Data::Agents(vec![agents::Observation {
            name: "x",
            version: None,
            executable: None,
        }]));
        app.data[9] = Some(Data::Developer(vec![agents::Observation {
            name: "x",
            version: None,
            executable: None,
        }]));
        app.data[7] = Some(Data::Remote(remote::status_with_tcp(
            |_| Err("error".into()),
            Err(()),
            Err(()),
        )));
        assert!(app.text().contains("Agents: desconocido"));
        assert!(app.text().contains("Developer: desconocido"));
        assert!(app.text().contains("Remote: desconocido"));
        app.data[9] = Some(Data::Developer(vec![agents::Observation {
            name: "x",
            version: Some("1.2.3".into()),
            executable: None,
        }]));
        assert!(app.text().contains("Developer: disponible"));
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 9,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        sender.send(Err("private".into())).unwrap();
        app.update();
        let text = app.text();
        assert!(text.contains("Developer: error (datos anteriores, pueden estar desactualizados)"));
        assert!(text.contains(query_error(9)));
        assert!(!text.contains("private"));
    }

    #[test]
    fn overview_all_views_evidence_boundaries_and_errors() {
        let observation = |version: Option<&str>, executable: Option<&str>| agents::Observation {
            name: "synthetic",
            version: version.map(str::to_owned),
            executable: executable.map(str::to_owned),
        };
        let update = |installed: Option<&str>, upstream: Option<&str>| updates::Observation {
            name: "synthetic",
            installed: installed.map(str::to_owned),
            upstream: upstream.map(str::to_owned),
            comparison: updates::Comparison::Unknown,
        };
        let unknown = vec![
            Data::Health(String::new(), false),
            Data::Services(vec![service("a")]),
            Data::Docker(vec![]),
            Data::Projects(vec![]),
            Data::Storage(storage::StorageStatus::default(), None),
            Data::Updates(vec![update(None, None)]),
            Data::Agents(vec![observation(None, None)]),
            Data::Remote(vec![]),
            Data::Watchdog(watchdog::Assessment::Unknown),
            Data::Developer(vec![observation(None, None)]),
        ];
        let present = vec![
            Data::Health("synthetic health".into(), true),
            Data::Services(vec![services::Service {
                name: "a".into(),
                load: Some("not-found".into()),
                active: None,
                sub: None,
            }]),
            Data::Docker(vec![docker::Container {
                name: "a".into(),
                id: "b".into(),
                image: "c".into(),
                state: "stopped".into(),
            }]),
            Data::Projects(vec![projects::Project {
                name: "a".into(),
                git: projects::GitState::Unknown,
                branch: None,
                head: None,
            }]),
            Data::Storage(
                storage::StorageStatus {
                    autofs: true,
                    ..Default::default()
                },
                None,
            ),
            Data::Updates(vec![update(None, Some("1.2.3"))]),
            Data::Agents(vec![observation(None, Some("synthetic"))]),
            Data::Remote(remote::status_with(|unit| {
                Ok(services::Service {
                    name: unit.into(),
                    load: Some("loaded".into()),
                    active: Some("active".into()),
                    sub: None,
                })
            })),
            Data::Watchdog(watchdog::Assessment::BelowMinimum),
            Data::Developer(vec![observation(Some("1.2.3"), None)]),
        ];
        let mut app = App::new();
        app.view = 10;
        for name in &NAMES[..10] {
            assert!(
                app.text().contains(&format!("{name}: no consultado")),
                "{name}"
            );
        }
        for (i, data) in unknown.into_iter().enumerate() {
            app.data[i] = Some(data);
            assert!(app.text().contains(&format!("{}: desconocido", NAMES[i])));
        }
        for (i, data) in present.into_iter().enumerate() {
            app.data[i] = Some(data);
            let expected = "disponible";
            assert!(app.text().contains(&format!("{}: {expected}", NAMES[i])));
            let before = format!("{:?}", app.data[i]);
            let (sender, receiver) = mpsc::channel();
            app.worker = Some(Worker {
                view: i,
                started: Instant::now(),
                receiver,
                timed_out: false,
            });
            sender.send(Err("private error".into())).unwrap();
            app.update();
            let text = app.text();
            assert!(text.contains(&format!(
                "{}: error (datos anteriores, pueden estar desactualizados) — {}",
                NAMES[i],
                query_error(i)
            )));
            assert!(!text.contains("private error"));
            assert_eq!(before, format!("{:?}", app.data[i]));
        }
    }

    fn service(name: &str) -> services::Service {
        services::Service {
            name: name.into(),
            load: None,
            active: None,
            sub: None,
        }
    }
    fn synthetic_storage_query(_: usize) -> Answer {
        Ok(Data::Storage(
            storage::StorageStatus {
                autofs: false,
                srv_ext4: true,
                docker_overlays: vec!["/srv/docker/overlay".into()],
            },
            None,
        ))
    }

    #[test]
    fn developer_manual_request_and_navigation() {
        fn fake(view: usize) -> Answer {
            assert_eq!(view, 9);
            Ok(Data::Developer(vec![agents::Observation {
                name: "Pi Agent",
                version: Some("1.2.3".into()),
                executable: None,
            }]))
        }
        fn fail(_: usize) -> Answer {
            Err("private error".into())
        }
        let mut app = App::new();
        for _ in 0..9 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 9);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Sin consultar"));
        app.key_with(KeyCode::Char('R'), fake);
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app.text().contains("versión npm=1.2.3"));
        assert!(app.text().contains("no vinculada al ejecutable"));
        app.key(KeyCode::Tab);
        app.key(KeyCode::Enter);
        assert!(app.detail);
        app.key(KeyCode::Esc);
        assert!(!app.detail);
        app.key_with(KeyCode::Char('r'), fail);
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.text().contains("Error al consultar Developer"));
        assert!(app.text().contains("versión npm=1.2.3"));
        assert!(!app.text().contains("private error"));
    }

    #[test]
    fn watchdog_manual_request_unknown_error_and_navigation() {
        fn fake(view: usize) -> Answer {
            assert_eq!(view, 8);
            Ok(Data::Watchdog(
                crate::modules::watchdog::Assessment::Unknown,
            ))
        }
        fn fail(_: usize) -> Answer {
            Err("private error".into())
        }
        let mut app = App::new();
        for _ in 0..8 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 8);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Sin consultar"));
        assert!(app.text().contains("20%"));
        assert!(app.text().contains("no diagnóstico"));
        app.key_with(KeyCode::Char('R'), fake);
        assert_eq!(app.worker.as_ref().map(|w| w.view), Some(8));
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app.text().contains("Resultado: desconocido"));
        app.key_with(KeyCode::Char('r'), fail);
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.text().contains("Error al consultar Watchdog"));
        assert!(app.text().contains("Resultado: desconocido"));
        assert!(!app.text().contains("private error"));
        assert!(app.worker.is_none());
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 9);
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 10);
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 11);
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 0);
        assert!(app.worker.is_none());
    }

    #[test]
    fn remote_manual_request_uses_worker() {
        fn fake(_: usize) -> Answer {
            Ok(Data::Remote(remote::status_with_tcp(
                |_| Err("privado".into()),
                Err(()),
                Err(()),
            )))
        }
        let mut app = App::new();
        for _ in 0..7 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 7);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Sin consultar"));
        app.key_with(KeyCode::Char('R'), fake);
        assert_eq!(app.worker.as_ref().map(|w| w.view), Some(7));
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app
            .text()
            .contains("SSH (ssh.service): carga=desconocido, actividad=desconocido"));
        assert!(app.text().contains("no demuestra conectividad"));
        assert!(app
            .text()
            .contains("TCP en escucha (sin atribución): desconocido"));
        assert!(app.text().contains("conectividad Tailscale=desconocido"));
        assert!(!app.text().contains("privado"));
        app.key(KeyCode::Tab);
        app.key(KeyCode::Enter);
        assert!(app.detail);
        app.key(KeyCode::Esc);
        assert!(!app.detail);
    }

    #[test]
    fn agents_manual_request_uses_worker_and_preserves_navigation() {
        fn fake(_: usize) -> Answer {
            Ok(Data::Agents(agents::status_with(|_| None)))
        }
        let mut app = App::new();
        for _ in 0..6 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 6);
        assert!(app.worker.is_none());
        app.key_with(KeyCode::Char('R'), fake);
        assert_eq!(app.worker.as_ref().map(|w| w.view), Some(6));
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app.text().contains(
            "Engram: ejecutable=desconocido, versión npm=desconocido (no vinculada al ejecutable), estado=desconocido"
        ));
        app.key(KeyCode::Tab);
        app.key(KeyCode::Enter);
        assert!(app.detail);
        app.key(KeyCode::Esc);
        assert!(!app.detail);
    }

    #[test]
    fn updates_manual_request_uses_worker_and_preserves_navigation() {
        fn fake(_: usize) -> Answer {
            Ok(Data::Updates(vec![updates::Observation {
                name: "Pi",
                installed: Some("1.2.3".into()),
                upstream: Some("2.0.0".into()),
                comparison: updates::Comparison::Available,
            }]))
        }
        let mut app = App::new();
        for _ in 0..5 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 5);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Sin consultar"));
        app.key_with(KeyCode::Char('R'), fake);
        assert_eq!(app.worker.as_ref().map(|w| w.view), Some(5));
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none());
        assert!(app
            .text()
            .contains("Pi: instalada=1.2.3, upstream=2.0.0, estado=actualización disponible"));
    }

    #[test]
    fn storage_capacity_rendering_is_exact() {
        let status = storage::StorageStatus::default();
        let capacity = storage::Capacity {
            total_bytes: 1000,
            used_bytes: 600,
            available_bytes: 300,
            reserved_bytes: 100,
        };
        let prefix = "Autofs en /srv: desconocido\nMontaje ext4 activo: desconocido\nPunto de montaje ext4: desconocido\nOverlays Docker reconocidos: 0\nCapacidad /srv: ";
        assert_eq!(
            Data::Storage(status, Some(capacity))
                .line(0, false)
                .unwrap(),
            format!("{prefix}total=1000 B, usada=600 B, disponible=300 B, reservada=100 B")
        );
        assert_eq!(
            Data::Storage(storage::StorageStatus::default(), None)
                .line(0, false)
                .unwrap(),
            format!("{prefix}desconocido")
        );
    }

    #[test]
    fn storage_is_manual_unknown_safe_and_preserves_navigation() {
        let mut app = App::new();
        for _ in 0..4 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 4);
        assert!(app.worker.is_none());
        assert!(app.text().contains("Sin consultar"));
        app.data[4] = Some(Data::Storage(storage::StorageStatus::default(), None));
        let text = app.text();
        assert!(text.contains("Autofs en /srv: desconocido"));
        assert!(text.contains("Montaje ext4 activo: desconocido"));
        assert!(text.contains("Punto de montaje ext4: desconocido"));
        assert!(text.contains("Overlays Docker reconocidos: 0"));
        app.data[4] = Some(Data::Storage(
            storage::StorageStatus {
                autofs: true,
                srv_ext4: true,
                docker_overlays: vec!["/srv/docker/a".into(), "/var/lib/docker/b".into()],
            },
            None,
        ));
        let text = app.text();
        assert!(text.contains("Autofs en /srv: identificado"));
        assert!(text.contains("Punto de montaje ext4: /srv"));
        assert!(text.contains("Overlays Docker reconocidos: 2"));
        app.data[4] = None;
        app.key_with(KeyCode::Char('R'), synthetic_storage_query);
        assert_eq!(app.worker.as_ref().map(|worker| worker.view), Some(4));
        let deadline = Instant::now() + Duration::from_secs(2);
        while app.worker.is_some() && Instant::now() < deadline {
            app.update();
            thread::yield_now();
        }
        assert!(app.worker.is_none(), "Storage query did not complete");
        assert!(app.text().contains("Overlays Docker reconocidos: 1"));
    }

    #[test]
    fn storage_query_error_is_view_specific_and_hides_details() {
        let mut app = App::new();
        app.view = 4;
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 4,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        sender.send(Err("raw parser detail".into())).unwrap();
        app.update();
        assert!(app
            .text()
            .contains("Error al consultar Storage (/proc/self/mountinfo)"));
        assert!(!app.text().contains("raw parser detail"));
    }

    #[test]
    fn selection_detail_and_empty_list() {
        let mut app = App::new();
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        app.key(KeyCode::Tab);
        app.data[1] = Some(Data::Services(vec![
            service("a.service"),
            service("b.service"),
        ]));
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert!(app.text().contains("Nombre: b.service"));
        assert!(app.text().contains("Categoría: desconocido"));
        app.key(KeyCode::Esc);
        assert!(app.text().contains("> b.service"));
        app.data[1] = Some(Data::Services(vec![]));
        app.key(KeyCode::Enter);
        assert!(!app.detail);
        assert!(app.text().contains("Sin elementos."));
        assert!(app.worker.is_none());
    }
    #[test]
    fn refresh_bounds_selection_and_keeps_worker_view() {
        let mut app = App::new();
        app.view = 1;
        app.table_focus = true;
        app.height = 2;
        app.data[1] = Some(Data::Services(
            (0..5).map(|_| service("a.service")).collect(),
        ));
        for _ in 0..4 {
            app.key(KeyCode::Down);
        }
        assert_eq!(app.row, 3);
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 1,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        sender
            .send(Ok(Data::Services(vec![service("nuevo.service")])))
            .unwrap();
        app.view = 2;
        app.update();
        assert_eq!(app.item[1], 0);
        assert!(app.data[2].is_none());
        assert!(app.data[1]
            .as_ref()
            .unwrap()
            .line(0, true)
            .unwrap()
            .contains("nuevo.service"));
    }
    #[test]
    fn details_use_only_selected_fields() {
        let docker = Data::Docker(vec![docker::Container {
            name: "demo".into(),
            id: "abc".into(),
            image: "image".into(),
            state: "running".into(),
        }]);
        assert_eq!(
            docker.line(0, true).unwrap(),
            "Nombre: demo\nIdentificador: abc\nImagen: image\nEstado: running"
        );
        let projects = Data::Projects(vec![projects::Project {
            name: "repo".into(),
            git: projects::GitState::Unknown,
            branch: None,
            head: None,
        }]);
        assert_eq!(
            projects.line(0, true).unwrap(),
            "Nombre: repo\nEvidencia Git: desconocido\nRama: desconocido\nHEAD: desconocido"
        );
    }
    #[test]
    fn selection_marker_survives_horizontal_scroll() {
        let mut app = App::new();
        app.view = 1;
        app.data[1] = Some(Data::Services(vec![service("long.service")]));
        app.column = 5;
        assert!(app.visible_content(1).starts_with("> "));
        app.column = 500;
        assert!(app.visible_content(1).starts_with("> "));
    }
    #[test]
    fn busy_request_preserves_other_view_results() {
        let mut app = App::new();
        let (_sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 0,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        app.view = 1;
        app.data[1] = Some(Data::Services(vec![service("old.service")]));
        app.request();
        assert!(app.text().contains("old.service"));
        assert!(app.notice.is_some());
        assert_eq!(app.worker.as_ref().unwrap().view, 0);
    }
    #[test]
    fn docker_and_projects_need_explicit_request() {
        let mut app = App::new();
        for _ in 0..2 {
            app.key(KeyCode::Down);
        }
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 2);
        assert!(app.worker.is_none());
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 3);
        assert!(app.worker.is_none());
    }
    #[test]
    fn scroll_is_bounded() {
        let mut app = App::new();
        app.content[0] = "primera\nsegunda fila larga\núltima".into();
        for _ in 0..100 {
            app.key(KeyCode::Char('j'));
            app.key(KeyCode::Char('l'));
        }
        assert_eq!(app.row, 2);
        assert_eq!(app.column, "segunda fila larga".chars().count() - 1);
        for _ in 0..100 {
            app.key(KeyCode::Char('k'));
            app.key(KeyCode::Char('h'));
        }
        assert_eq!((app.row, app.column), (0, 0));
    }
    #[test]
    fn background_errors_and_timeout_remain_on_origin() {
        let mut app = App::new();
        app.view = 2;
        app.data[1] = Some(Data::Services(vec![service("old.service")]));
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 1,
            started: Instant::now() - LIMIT,
            receiver,
            timed_out: false,
        });
        app.update();
        assert!(app.status[1].as_ref().unwrap().contains("diez segundos"));
        assert!(app.status[2].is_none());
        sender.send(Err("falló".into())).unwrap();
        app.update();
        assert_eq!(
            app.status[1].as_deref(),
            Some("Error al consultar servicios (systemd); datos anteriores")
        );
        assert!(app.status[2].is_none());
        app.view = 1;
        assert!(app.text().contains("old.service"));
        assert!(app.status[app.view]
            .as_ref()
            .unwrap()
            .contains("datos anteriores"));
    }
    #[test]
    fn successful_refresh_closes_detail_before_reordering() {
        let mut app = App::new();
        app.view = 1;
        app.table_focus = true;
        app.data[1] = Some(Data::Services(vec![service("old.service")]));
        app.key(KeyCode::Enter);
        assert!(app.detail);
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 1,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        sender
            .send(Ok(Data::Services(vec![service("different.service")])))
            .unwrap();
        app.update();
        assert!(!app.detail);
        assert!(app.text().contains("different.service"));
    }
    #[test]
    fn detail_resets_and_restores_horizontal_scroll() {
        let mut app = App::new();
        app.view = 1;
        app.table_focus = true;
        app.data[1] = Some(Data::Services(vec![service("long.service")]));
        app.column = 4;
        app.key(KeyCode::Enter);
        assert_eq!(app.column, 0);
        assert!(app.visible_content(1).starts_with("Nombre:"));
        app.key(KeyCode::Esc);
        assert_eq!(app.column, 4);
    }
    #[test]
    fn returning_to_view_and_moving_selection_restores_visibility() {
        let mut app = App::new();
        app.view = 1;
        app.selected = 1;
        app.height = 2;
        app.data[1] = Some(Data::Services(
            (0..8).map(|_| service("a.service")).collect(),
        ));
        app.item[1] = 7;
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        app.key(KeyCode::Up);
        app.key(KeyCode::Enter);
        assert_eq!(app.view, 1);
        assert_eq!(app.row, 6);
        app.key(KeyCode::Tab);
        app.key(KeyCode::Char('k'));
        assert_eq!(app.row, 6);
        app.key(KeyCode::Up);
        assert!(app.row <= 6 && 6 < app.row + app.height);
    }
    #[test]
    fn status_is_scrollable_and_disconnect_clears_busy_notice() {
        let mut app = App::new();
        app.view = 1;
        app.data[1] = Some(Data::Services(vec![service("a.service")]));
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 1,
            started: Instant::now() - LIMIT,
            receiver,
            timed_out: false,
        });
        app.update();
        assert!(app.text().contains("no cancelada"));
        assert!(app.text().contains("no es un inventario completo"));
        app.request();
        assert!(app.notice.is_some());
        drop(sender);
        app.update();
        assert!(app.notice.is_none());
        assert!(app.text().contains("consulta terminó inesperadamente"));
        app.row = app.text().lines().count() - 1;
        app.column = 0;
        assert!(app.visible_content(1).contains("inesperadamente"));
    }
    #[test]
    fn service_error_does_not_expose_untrusted_stderr() {
        let mut app = App::new();
        app.view = 1;
        app.data[1] = Some(Data::Services(vec![service("old.service")]));
        let (sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 1,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        sender
            .send(Err("SECRET_SENTINEL from stderr".into()))
            .unwrap();
        app.update();
        assert!(!app.text().contains("SECRET_SENTINEL"));
        assert!(!app.status[1].as_ref().unwrap().contains("SECRET_SENTINEL"));
        assert!(app
            .text()
            .contains("Error al consultar servicios (systemd)"));
        assert!(app.text().contains("old.service"));
    }
    #[test]
    fn busy_notice_is_scrollable_and_table_scroll_keeps_selection() {
        let mut app = App::new();
        app.view = 1;
        app.height = 2;
        app.table_focus = true;
        app.data[1] = Some(Data::Services(
            (0..5).map(|_| service("a.service")).collect(),
        ));
        let (_sender, receiver) = mpsc::channel();
        app.worker = Some(Worker {
            view: 0,
            started: Instant::now(),
            receiver,
            timed_out: false,
        });
        app.request();
        assert!(app.text().contains("Hay una consulta en curso"));
        for _ in 0..20 {
            app.key(KeyCode::Char('j'));
        }
        assert!(app.row <= app.item[1] && app.item[1] < app.row + app.height);
        app.key(KeyCode::Tab);
        for _ in 0..100 {
            app.key(KeyCode::Char('j'));
        }
        assert!(app.visible_content(1).contains("Hay una consulta en curso"));
    }
    #[test]
    fn menu_and_scroll_preserved() {
        let mut app = App::new();
        app.content[0] = "primera\nsegunda fila larga\núltima".into();
        app.key(KeyCode::Char('j'));
        app.key(KeyCode::Char('l'));
        assert_eq!(app.visible_content(1), "egunda fila larga");
        app.key(KeyCode::Down);
        app.key(KeyCode::Enter);
        assert_eq!((app.view, app.row, app.column), (1, 0, 0));
        assert!(app.worker.is_none());
        assert!(!app.key(KeyCode::Esc));
    }
}
