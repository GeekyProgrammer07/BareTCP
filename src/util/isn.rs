use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn generate_isn() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::new(0, 0))
        .as_micros() as u32
}
