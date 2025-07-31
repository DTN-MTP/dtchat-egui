use chrono::{DateTime, Utc};

pub fn ts_to_str(
    datetime: &DateTime<Utc>,
    date: bool,
    time: bool,
    separator: Option<String>,
) -> String {
    let mut res = "".to_string();
    if date {
        res += &datetime.format("%Y-%m-%d").to_string();
    }
    if let Some(sep) = separator {
        res += &sep;
    }
    if time {
        res += &datetime.format("%H:%M:%S").to_string()
    }
    res
}

pub fn clock(hours: u32, mins: u32) -> char {
    let mut idx: u32 = hours;
    if mins > 45 {
        idx += 1;
    }

    if idx == 0 {
        idx = 12;
    }

    if mins > 15 && mins < 45 {
        idx += 12;
    };
    char::from_u32(0x1F54F + idx).unwrap()
}
