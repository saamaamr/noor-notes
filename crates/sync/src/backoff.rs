use std::time::Duration;

pub fn retry_delay(attempt: u32) -> Duration {
    const SECONDS: [u64; 5] = [2, 4, 8, 16, 30];
    Duration::from_secs(SECONDS[attempt.min(4) as usize])
}
