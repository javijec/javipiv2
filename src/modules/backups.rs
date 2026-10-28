//! Observación de datos JSON suministrados explícitamente; no accede a Restic.
use serde_json::Value;

pub struct Snapshot {
    pub time: String,
    pub id: Option<String>,
    pub profile: Option<String>,
}

pub struct Observation {
    pub count: usize,
    pub newest: Option<Snapshot>,
}

// Compara segundos UTC y fracciones con precisión arbitraria, sin depender del orden JSON.
fn instant(s: &str) -> Option<(i64, String)> {
    let b = s.as_bytes();
    if b.len() < 20
        || b.get(4) != Some(&b'-')
        || b.get(7) != Some(&b'-')
        || !matches!(b.get(10), Some(b'T' | b't'))
        || b.get(13) != Some(&b':')
        || b.get(16) != Some(&b':')
    {
        return None;
    }
    let number = |start: usize, end: usize| -> Option<i64> {
        let part = b.get(start..end)?;
        if !part.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(part).ok()?.parse().ok()
    };
    let (year, month, day, hour, minute, second) = (
        number(0, 4)?,
        number(5, 7)?,
        number(8, 10)?,
        number(11, 13)?,
        number(14, 16)?,
        number(17, 19)?,
    );
    if !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day < 1 || day > days[(month - 1) as usize] {
        return None;
    }
    let mut pos = 19;
    let mut fraction = String::new();
    if b.get(pos) == Some(&b'.') {
        pos += 1;
        let start = pos;
        while b.get(pos).is_some_and(u8::is_ascii_digit) {
            pos += 1;
        }
        if pos == start {
            return None;
        }
        fraction = s.get(start..pos)?.trim_end_matches('0').to_owned();
    }
    let offset = match b.get(pos) {
        Some(b'Z' | b'z') if pos + 1 == b.len() => 0,
        Some(sign @ (b'+' | b'-')) if pos + 6 == b.len() && b.get(pos + 3) == Some(&b':') => {
            let h = number(pos + 1, pos + 3)?;
            let m = number(pos + 4, pos + 6)?;
            if h > 23 || m > 59 || (*sign == b'-' && h == 0 && m == 0) {
                return None;
            }
            (h * 60 + m) * 60 * if *sign == b'+' { 1 } else { -1 }
        }
        _ => return None,
    };
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let date = era * 146097 + yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some((
        date * 86400 + hour * 3600 + minute * 60 + second - offset,
        fraction,
    ))
}

pub fn inspect(input: &[u8]) -> Result<Observation, &'static str> {
    let value: Value = serde_json::from_slice(input).map_err(|_| "JSON de snapshots inválido")?;
    let entries = value.as_array().ok_or("JSON de snapshots inválido")?;
    let mut latest: Option<((i64, String), &Value)> = None;
    let mut invalid_date = false;
    let mut tied = false;
    for entry in entries {
        let Some(key) = entry.get("time").and_then(Value::as_str).and_then(instant) else {
            invalid_date = true;
            continue;
        };
        match &latest {
            Some((old, _)) if key == *old => tied = true,
            Some((old, _)) if key < *old => (),
            _ => {
                latest = Some((key, entry));
                tied = false;
            }
        }
    }
    let newest = if invalid_date || tied {
        None
    } else {
        latest.map(|(_, v)| {
            let id = ["short_id", "id"]
                .into_iter()
                .filter_map(|field| v.get(field).and_then(Value::as_str))
                .find(|s| s.len() >= 8 && s.bytes().all(|c| c.is_ascii_hexdigit()))
                .map(|s| s.chars().take(8).collect());
            let mut profiles = v
                .get("tags")
                .and_then(Value::as_array)
                .filter(|tags| tags.iter().all(Value::is_string))
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter_map(|s| match s {
                    "profile:full" => Some("full"),
                    "profile:projects" => Some("projects"),
                    "profile:engram" => Some("engram"),
                    _ => None,
                });
            let first = profiles.next();
            let profile = first
                .filter(|p| profiles.all(|other| other == *p))
                .map(str::to_owned);
            Snapshot {
                time: v["time"].as_str().unwrap_or_default().to_owned(),
                id,
                profile,
            }
        })
    };
    Ok(Observation {
        count: entries.len(),
        newest,
    })
}
