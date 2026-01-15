use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Metadata {
    pub created_at_ms: u64,
    pub flags: u32,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            created_at_ms: now_unix_ms(),
            flags: 0,
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
