use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Timestamp utilities for async operations
pub mod timestamp {
    use super::*;

    /// Format the current time as HH:MM:SS (24-hour format)
    pub fn format_current_time() -> String {
        format_time(SystemTime::now())
    }

    /// Format a SystemTime as HH:MM:SS (24-hour format)
    pub fn format_time(time: SystemTime) -> String {
        let duration_since_epoch = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));

        let total_seconds = duration_since_epoch.as_secs();
        let hours = (total_seconds / 3600) % 24;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }

    /// Calculate the duration between two Instants and return as rounded seconds
    pub fn duration_as_rounded_seconds(start: Instant, end: Instant) -> u64 {
        end.duration_since(start).as_secs()
    }

    /// Calculate the duration from an Instant to now and return as rounded seconds
    pub fn duration_since_as_rounded_seconds(start: Instant) -> u64 {
        duration_as_rounded_seconds(start, Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_format_time_basic() {
        let time = UNIX_EPOCH + Duration::from_secs(3661); // 1 hour, 1 minute, 1 second
        let formatted = timestamp::format_time(time);
        assert_eq!(formatted, "01:01:01");
    }

    #[test]
    fn test_format_time_midnight() {
        let time = UNIX_EPOCH + Duration::from_secs(0);
        let formatted = timestamp::format_time(time);
        assert_eq!(formatted, "00:00:00");
    }

    #[test]
    fn test_format_time_end_of_day() {
        let time = UNIX_EPOCH + Duration::from_secs(86399); // 23:59:59
        let formatted = timestamp::format_time(time);
        assert_eq!(formatted, "23:59:59");
    }

    #[test]
    fn test_format_current_time_is_valid_format() {
        let formatted = timestamp::format_current_time();
        assert!(formatted.len() == 8); // HH:MM:SS format
        assert!(formatted.chars().nth(2) == Some(':'));
        assert!(formatted.chars().nth(5) == Some(':'));
    }

    #[test]
    fn test_duration_as_rounded_seconds() {
        let start = Instant::now();
        thread::sleep(Duration::from_millis(1100)); // Just over 1 second
        let end = Instant::now();

        let duration = timestamp::duration_as_rounded_seconds(start, end);
        assert_eq!(duration, 1); // Should round down to 1 second
    }

    #[test]
    fn test_duration_since_as_rounded_seconds() {
        let start = Instant::now();
        thread::sleep(Duration::from_millis(500)); // Half a second

        let duration = timestamp::duration_since_as_rounded_seconds(start);
        assert_eq!(duration, 0); // Should round down to 0 seconds
    }

    #[test]
    fn test_duration_zero() {
        let instant = Instant::now();
        let duration = timestamp::duration_as_rounded_seconds(instant, instant);
        assert_eq!(duration, 0);
    }
}
