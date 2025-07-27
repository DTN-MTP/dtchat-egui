use chrono::{DateTime, Utc};

pub fn f64_to_utc(timestamp: f64) -> DateTime<Utc> {
    let secs = timestamp.trunc() as i64;
    let nsecs = ((timestamp.fract()) * 1_000_000_000.0).round() as u32;
    let naive = DateTime::from_timestamp(secs, nsecs).expect("Invalid timestamp");
    DateTime::from_naive_utc_and_offset(naive.naive_utc(), Utc)
}
