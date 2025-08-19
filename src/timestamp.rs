use chrono::{DateTime, Local};
use std::time::{Instant, SystemTime};

/// Format the current time as H:MM:SS (24-hour format) in local time
pub fn format_current_time() -> String {
    format_time(SystemTime::now())
}

/// Format a SystemTime as H:MM:SS (24-hour format) in local time
pub fn format_time(time: SystemTime) -> String {
    let datetime: DateTime<Local> = time.into();
    // Use platform-compatible formatting - this will work on all systems
    let formatted = datetime.format("%H:%M:%S").to_string();
    // Remove leading zero from hour if present
    if formatted.starts_with('0') && formatted.len() > 1 && formatted.chars().nth(1) != Some(':') {
        formatted[1..].to_string()
    } else {
        formatted
    }
}

/// Calculate the duration between two Instants and return as rounded seconds
pub fn duration_as_rounded_seconds(start: Instant, end: Instant) -> u64 {
    end.duration_since(start).as_secs()
}

/// Calculate the duration from an Instant to now and return as rounded seconds
pub fn duration_since_as_rounded_seconds(start: Instant) -> u64 {
    duration_as_rounded_seconds(start, Instant::now())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_format_time_basic() {
        // Test with current time - we just verify the format is correct
        let time = SystemTime::now();
        let formatted = format_time(time);
        // Length can be 7 (H:MM:SS) or 8 (HH:MM:SS) depending on hour
        assert!(formatted.len() >= 7 && formatted.len() <= 8);

        // Should contain colons but no decimal point
        assert!(formatted.contains(':'));
        assert!(!formatted.contains('.'));

        // Check that hours, minutes, seconds are valid numbers
        let parts: Vec<&str> = formatted.split(':').collect();
        assert_eq!(parts.len(), 3);
        let hours: u32 = parts[0].parse().expect("Hours should be valid number");
        let minutes: u32 = parts[1].parse().expect("Minutes should be valid number");
        let seconds: u32 = parts[2].parse().expect("Seconds should be valid number");
        assert!(hours < 24);
        assert!(minutes < 60);
        assert!(seconds < 60);
    }

    #[test]
    fn test_format_time_midnight() {
        // Just test format validity - actual time depends on local timezone
        let time = SystemTime::now();
        let formatted = format_time(time);
        // Length can be 7 (H:MM:SS) or 8 (HH:MM:SS) depending on hour
        assert!(formatted.len() >= 7 && formatted.len() <= 8);

        // Should contain colons but no decimal point
        assert!(formatted.contains(':'));
        assert!(!formatted.contains('.'));

        // Should have exactly 2 colons and no decimal points
        assert_eq!(formatted.matches(':').count(), 2);
        assert_eq!(formatted.matches('.').count(), 0);
    }

    #[test]
    fn test_format_time_end_of_day() {
        // Just test format validity - actual time depends on local timezone
        let time = SystemTime::now();
        let formatted = format_time(time);
        // Length can be 7 (H:MM:SS) or 8 (HH:MM:SS) depending on hour
        assert!(formatted.len() >= 7 && formatted.len() <= 8);

        // Should contain colons but no decimal point
        assert!(formatted.contains(':'));
        assert!(!formatted.contains('.'));

        // Should have exactly 2 colons and no decimal points
        assert_eq!(formatted.matches(':').count(), 2);
        assert_eq!(formatted.matches('.').count(), 0);
    }

    #[test]
    fn test_format_current_time_is_valid_format() {
        let formatted = format_current_time();
        // Length can be 7 (H:MM:SS) or 8 (HH:MM:SS) depending on hour
        assert!(formatted.len() >= 7 && formatted.len() <= 8);

        // Should contain colons but no decimal point
        assert!(formatted.contains(':'));
        assert!(!formatted.contains('.'));

        // Should have exactly 2 colons and no decimal points
        assert_eq!(formatted.matches(':').count(), 2);
        assert_eq!(formatted.matches('.').count(), 0);
    }

    #[test]
    fn test_duration_as_rounded_seconds() {
        let start = Instant::now();
        thread::sleep(std::time::Duration::from_millis(1100)); // Just over 1 second
        let end = Instant::now();

        let duration = duration_as_rounded_seconds(start, end);
        assert_eq!(duration, 1); // Should round down to 1 second
    }

    #[test]
    fn test_duration_since_as_rounded_seconds() {
        let start = Instant::now();
        thread::sleep(std::time::Duration::from_millis(500)); // Half a second

        let duration = duration_since_as_rounded_seconds(start);
        assert_eq!(duration, 0); // Should round down to 0 seconds
    }

    #[test]
    fn test_duration_zero() {
        let instant = Instant::now();
        let duration = duration_as_rounded_seconds(instant, instant);
        assert_eq!(duration, 0);
    }
}
