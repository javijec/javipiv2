//! Observación local de unidades conocidas, sin inferir conectividad.
use super::services;
use std::io::Read;

const MAX_TCP: usize = 1024 * 1024;

fn parse_tcp(data: &[u8], address_len: usize) -> Option<bool> {
    if data.len() > MAX_TCP || !data.ends_with(b"\n") {
        return None;
    }
    let text = std::str::from_utf8(data).ok()?;
    let mut lines = text.lines();
    let mut header: Vec<_> = lines.next()?.split_whitespace().collect();
    if address_len == 32 && header.get(2) == Some(&"remote_address") {
        header[2] = "rem_address";
    }
    if header
        != [
            "sl",
            "local_address",
            "rem_address",
            "st",
            "tx_queue",
            "rx_queue",
            "tr",
            "tm->when",
            "retrnsmt",
            "uid",
            "timeout",
            "inode",
        ]
    {
        return None;
    }
    let mut listen = false;
    for line in lines {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 10 {
            return None;
        }
        let index = fields[0].strip_suffix(':')?;
        if index.is_empty() || !index.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        for field in &fields[1..3] {
            let (addr, port) = field.split_once(':')?;
            if addr.len() != address_len
                || port.len() != 4
                || !addr
                    .bytes()
                    .chain(port.bytes())
                    .all(|b| b.is_ascii_hexdigit())
            {
                return None;
            }
        }
        let hex = |s: &str, n: usize| s.len() == n && s.bytes().all(|b| b.is_ascii_hexdigit());
        let pair = |s: &str, a: usize, b: usize| {
            s.split_once(':')
                .is_some_and(|(x, y)| hex(x, a) && hex(y, b))
        };
        let decimal = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if !hex(fields[3], 2)
            || !pair(fields[4], 8, 8)
            || !pair(fields[5], 2, 8)
            || !hex(fields[6], 8)
            || !fields[7..10].iter().all(|s| decimal(s))
        {
            return None;
        }
        listen |= fields[3] == "0A";
    }
    Some(listen)
}

fn tcp_evidence(v4: Result<&[u8], ()>, v6: Result<&[u8], ()>) -> Option<bool> {
    let (v4, v6) = v4
        .ok()
        .and_then(|data| parse_tcp(data, 8))
        .zip(v6.ok().and_then(|data| parse_tcp(data, 32)))?;
    Some(v4 || v6)
}

pub fn tcp_line_with(v4: Result<&[u8], ()>, v6: Result<&[u8], ()>) -> String {
    let value = match tcp_evidence(v4, v6) {
        Some(true) => "sí",
        Some(false) => "no",
        None => "desconocido",
    };
    format!("TCP en escucha (sin atribución): {value}; SSH en escucha=desconocido; conectividad SSH=desconocido; conectividad Tailscale=desconocido")
}

fn read_tcp(path: &str) -> Result<Vec<u8>, ()> {
    let mut data = Vec::new();
    std::fs::File::open(path)
        .map_err(|_| ())?
        .take((MAX_TCP + 1) as u64)
        .read_to_end(&mut data)
        .map_err(|_| ())?;
    Ok(data)
}

pub fn tcp_line() -> String {
    let v4 = read_tcp("/proc/net/tcp");
    let v6 = read_tcp("/proc/net/tcp6");
    tcp_line_with(v4.as_deref().map_err(|_| ()), v6.as_deref().map_err(|_| ()))
}

#[derive(Debug)]
pub struct Observation {
    name: &'static str,
    unit: &'static str,
    load: Option<String>,
    active: Option<String>,
    extra: Option<String>,
    tcp: Option<bool>,
}

impl Observation {
    pub fn has_evidence(&self) -> bool {
        let known = |value: &Option<String>| {
            value
                .as_deref()
                .is_some_and(|s| !s.is_empty() && s != "-" && s != "desconocido")
        };
        known(&self.load) || known(&self.active) || self.tcp.is_some()
    }

    pub fn line(&self) -> String {
        if let Some(extra) = &self.extra {
            return extra.clone();
        }
        format!(
            "{} ({}): carga={}, actividad={}",
            self.name,
            self.unit,
            self.load.as_deref().unwrap_or("desconocido"),
            self.active.as_deref().unwrap_or("desconocido")
        )
    }
}

pub fn status_with(
    mut query: impl FnMut(&str) -> Result<services::Service, String>,
) -> Vec<Observation> {
    [("SSH", "ssh.service"), ("Tailscale", "tailscaled.service")]
        .into_iter()
        .map(|(name, unit)| {
            let result = query(unit).ok().filter(|service| service.name == unit);
            Observation {
                name,
                unit,
                extra: None,
                tcp: None,
                load: result
                    .as_ref()
                    .and_then(|s| s.load.clone())
                    .filter(|s| !s.is_empty() && s != "-"),
                active: result
                    .and_then(|s| s.active)
                    .filter(|s| !s.is_empty() && s != "-"),
            }
        })
        .collect()
}

pub fn status_with_tcp(
    query: impl FnMut(&str) -> Result<services::Service, String>,
    v4: Result<&[u8], ()>,
    v6: Result<&[u8], ()>,
) -> Vec<Observation> {
    let mut observations = status_with(query);
    observations.push(Observation {
        name: "",
        unit: "",
        load: None,
        active: None,
        extra: Some(tcp_line_with(v4, v6)),
        tcp: tcp_evidence(v4, v6),
    });
    observations
}

pub fn status() -> Vec<Observation> {
    let mut observations = status_with(services::status);
    let v4 = read_tcp("/proc/net/tcp");
    let v6 = read_tcp("/proc/net/tcp6");
    let v4 = v4.as_deref().map_err(|_| ());
    let v6 = v6.as_deref().map_err(|_| ());
    observations.push(Observation {
        name: "",
        unit: "",
        load: None,
        active: None,
        extra: Some(tcp_line_with(v4, v6)),
        tcp: tcp_evidence(v4, v6),
    });
    observations
}
