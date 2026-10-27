//! Lecturas acotadas de pseudoarchivos del kernel.
//! No utiliza /srv, sudo, systemctl, Docker ni ejecuta subprocesses.

use std::fs;
use std::io;

#[derive(Debug, PartialEq, Eq)]
pub struct SystemStatus {
    pub hostname: String,
    pub uptime_seconds: u64,
    pub memory_total_kib: u64,
    pub memory_available_kib: u64,
}

pub fn parse_uptime(raw: &str) -> Option<u64> {
    let seconds = raw.split_whitespace().next()?;
    let (integer, fraction) = seconds.split_once('.')?;
    if integer.is_empty()
        || fraction.is_empty()
        || !integer.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    integer.parse().ok()
}

pub fn parse_memory_kib(raw: &str, key: &str) -> Option<u64> {
    raw.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name != key {
            return None;
        }
        let mut fields = value.split_whitespace();
        match (
            fields.next()?.parse::<u64>().ok(),
            fields.next(),
            fields.next(),
        ) {
            (Some(amount), Some("kB"), None) => Some(amount),
            _ => None,
        }
    })
}

pub fn parse_status(host: &str, uptime: &str, meminfo: &str) -> io::Result<SystemStatus> {
    let invalid = |label| io::Error::new(io::ErrorKind::InvalidData, label);
    let hostname = host.trim();
    if hostname.is_empty() || hostname.chars().any(char::is_control) {
        return Err(invalid("hostname inválido"));
    }
    let seconds = parse_uptime(uptime).ok_or_else(|| invalid("uptime inválido"))?;
    let total = parse_memory_kib(meminfo, "MemTotal")
        .ok_or_else(|| invalid("MemTotal inválido"))?;
    let available = parse_memory_kib(meminfo, "MemAvailable")
        .ok_or_else(|| invalid("MemAvailable inválido"))?;
    if total == 0 || available > total {
        return Err(invalid("valores de memoria incoherentes"));
    }
    Ok(SystemStatus {
        hostname: hostname.to_owned(),
        uptime_seconds: seconds,
        memory_total_kib: total,
        memory_available_kib: available,
    })
}

pub fn read_status() -> io::Result<SystemStatus> {
    let host = fs::read_to_string("/proc/sys/kernel/hostname")?;
    let uptime = fs::read_to_string("/proc/uptime")?;
    let meminfo = fs::read_to_string("/proc/meminfo")?;
    parse_status(&host, &uptime, &meminfo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_parses_integer_seconds() {
        assert_eq!(parse_uptime("1234.56 1024.00"), Some(1234));
        assert_eq!(parse_uptime("-1.5 0"), None);
        assert_eq!(parse_uptime("1234 1024"), None);
        assert_eq!(parse_uptime("1234.no 0"), None);
    }

    #[test]
    fn memory_fields_are_exact() {
        let raw = "MemTotal:       2048000 kB\nMemAvailable: 512000 kB\n";
        assert_eq!(parse_memory_kib(raw, "MemTotal"), Some(2_048_000));
        assert_eq!(parse_memory_kib(raw, "MemAvailable"), Some(512_000));
        assert_eq!(parse_memory_kib(raw, "MemFree"), None);
        assert_eq!(parse_memory_kib("MemTotal: 200 MB", "MemTotal"), None);
    }

    #[test]
    fn status_rejects_impossible_memory() {
        assert!(parse_status("pi\n", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 70 kB").is_ok());
        assert!(parse_status("pi", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 120 kB").is_err());
        assert!(parse_status("\n", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 70 kB").is_err());
    }
}
