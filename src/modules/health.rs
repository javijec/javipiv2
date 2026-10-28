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

#[derive(Debug, PartialEq, Eq)]
pub struct CpuObservation {
    /// Kernel load averages over 1, 5 and 15 minutes, if available.
    pub load_average: Option<[String; 3]>,
    /// CPU temperature in millidegrees Celsius, if uniquely identified.
    pub cpu_temperature_millicelsius: Option<i64>,
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

pub fn parse_load_average(raw: &str) -> Option<[String; 3]> {
    let mut fields = raw.split_whitespace();
    let mut read_value = || {
        let value = fields.next()?;
        let number = value.parse::<f64>().ok()?;
        (number.is_finite() && number >= 0.0).then(|| value.to_owned())
    };
    Some([read_value()?, read_value()?, read_value()?])
}

/// Accept only one explicitly identified CPU thermal zone with a valid reading.
pub fn parse_cpu_temperature(zones: &[(String, Option<String>)]) -> Option<i64> {
    let mut cpu_zones = zones
        .iter()
        .filter(|(kind, _)| kind.trim() == "cpu-thermal");
    let (_, temperature) = cpu_zones.next()?;
    if cpu_zones.next().is_some() {
        return None;
    }
    temperature.as_deref()?.trim().parse().ok()
}

fn read_cpu_temperature() -> Option<i64> {
    let mut zones = Vec::new();
    for entry in fs::read_dir("/sys/class/thermal").ok()? {
        let entry = entry.ok()?;
        let name = entry.file_name();
        let name = name.to_str()?;
        let Some(suffix) = name.strip_prefix("thermal_zone") else {
            continue;
        };
        if suffix.is_empty() || !suffix.bytes().all(|b| b.is_ascii_digit()) {
            continue;
        }
        let kind = fs::read_to_string(entry.path().join("type")).ok()?;
        let temperature = if kind.trim() == "cpu-thermal" {
            fs::read_to_string(entry.path().join("temp")).ok()
        } else {
            None
        };
        zones.push((kind, temperature));
    }
    parse_cpu_temperature(&zones)
}

pub fn parse_status(host: &str, uptime: &str, meminfo: &str) -> io::Result<SystemStatus> {
    let invalid = |label| io::Error::new(io::ErrorKind::InvalidData, label);
    let hostname = host.trim();
    if hostname.is_empty() || hostname.chars().any(char::is_control) {
        return Err(invalid("hostname inválido"));
    }
    let seconds = parse_uptime(uptime).ok_or_else(|| invalid("uptime inválido"))?;
    let total =
        parse_memory_kib(meminfo, "MemTotal").ok_or_else(|| invalid("MemTotal inválido"))?;
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

pub fn read_cpu_observation() -> CpuObservation {
    CpuObservation {
        load_average: fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|raw| parse_load_average(&raw)),
        cpu_temperature_millicelsius: read_cpu_temperature(),
    }
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
    fn load_averages_require_three_finite_nonnegative_values() {
        assert_eq!(
            parse_load_average("0.25 1.50 2.75 1/100 123\n"),
            Some(["0.25".into(), "1.50".into(), "2.75".into()])
        );
        assert_eq!(parse_load_average("0.25 1.50"), None);
        assert_eq!(parse_load_average("0.25 NaN 2.75"), None);
        assert_eq!(parse_load_average("0.25 -1 2.75"), None);
    }

    #[test]
    fn temperature_requires_one_identified_readable_cpu_zone() {
        let cpu = ("cpu-thermal\n".into(), Some("42500\n".into()));
        let other = ("gpu-thermal".into(), Some("50000".into()));
        assert_eq!(parse_cpu_temperature(&[other, cpu.clone()]), Some(42_500));
        assert_eq!(parse_cpu_temperature(&[]), None);
        assert_eq!(parse_cpu_temperature(&[cpu.clone(), cpu]), None);
        assert_eq!(parse_cpu_temperature(&[("cpu-thermal".into(), None)]), None);
        assert_eq!(
            parse_cpu_temperature(&[("cpu-thermal".into(), Some("invalid".into()))]),
            None
        );
        assert_eq!(
            parse_cpu_temperature(&[("unknown".into(), Some("42500".into()))]),
            None
        );
    }

    #[test]
    fn status_rejects_impossible_memory() {
        let status =
            parse_status("pi\n", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 70 kB").unwrap();
        assert_eq!(status.hostname, "pi");
        assert_eq!(status.uptime_seconds, 20);
        assert_eq!(status.memory_total_kib, 100);
        assert_eq!(status.memory_available_kib, 70);
        assert!(parse_status("pi", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 120 kB").is_err());
        assert!(parse_status("\n", "20.1 4.5", "MemTotal: 100 kB\nMemAvailable: 70 kB").is_err());
    }
}
