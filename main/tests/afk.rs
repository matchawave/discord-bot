#[cfg(test)]
use chrono::{DateTime, Duration, Utc};

/// This module contains tests for the `calculate_duration` function, which formats the duration of a user's AFK status into a human-readable string. The tests cover various scenarios, including durations that are only seconds, only minutes, only hours, and combinations of these units. The tests also check for correct singular and plural forms of the time units.
/// You can find the `calculate_duration` function in the `main/src/events/message/afk.rs` file, which is responsible for calculating the duration of a user's AFK status and formatting it as a string that indicates how long the user was away. The tests ensure that the function behaves correctly across a range of input durations.
fn calculate_duration(created_at: DateTime<Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now.signed_duration_since(created_at);
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    let seconds = duration.num_seconds() % 60;

    let mut output_time = Vec::new();
    if hours > 0 {
        output_time.push(format!("{} hr{}", hours, if hours == 1 { "" } else { "s" }));
    }
    if minutes > 0 {
        output_time.push(format!(
            "{} min{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if output_time.len() < 2 && seconds > 0 {
        output_time.push(format!(
            "{} sec{}",
            seconds,
            if seconds == 1 { "" } else { "s" }
        ));
    }

    output_time.join(" ")
}

#[test]
fn test_seconds_only() {
    let now = Utc::now();
    let past = now - Duration::seconds(45);
    let result = calculate_duration(past);
    assert!(result.contains("sec"), "Expected 'sec' in: {}", result);
    assert!(result.contains("45"), "Expected '45' in: {}", result);
}

#[test]
fn test_one_second() {
    let now = Utc::now();
    let past = now - Duration::seconds(1);
    let result = calculate_duration(past);
    assert_eq!(result, "1 sec", "Singular 'sec' expected: {}", result);
}

#[test]
fn test_minutes_only() {
    let now = Utc::now();
    let past = now - Duration::minutes(15);
    let result = calculate_duration(past);
    assert!(result.contains("min"), "Expected 'min' in: {}", result);
    assert!(result.contains("15"), "Expected '15' in: {}", result);
}

#[test]
fn test_one_minute() {
    let now = Utc::now();
    let past = now - Duration::minutes(1);
    let result = calculate_duration(past);
    assert_eq!(result, "1 min", "Singular 'min' expected: {}", result);
}

#[test]
fn test_minutes_and_seconds() {
    let now = Utc::now();
    let past = now - Duration::minutes(5) - Duration::seconds(30);
    let result = calculate_duration(past);
    assert!(result.contains("min"), "Expected 'min' in: {}", result);
    assert!(
        result.contains("sec"),
        "Should include seconds with minutes: {}",
        result
    );
}

#[test]
fn test_hours_only() {
    let now = Utc::now();
    let past = now - Duration::hours(3);
    let result = calculate_duration(past);
    assert!(result.contains("hr"), "Expected 'hr' in: {}", result);
    assert!(result.contains("3"), "Expected '3' in: {}", result);
}

#[test]
fn test_one_hour() {
    let now = Utc::now();
    let past = now - Duration::hours(1);
    let result = calculate_duration(past);
    assert!(
        result.starts_with("1 hr"),
        "Singular 'hr' expected: {}",
        result
    );
}

#[test]
fn test_hours_and_minutes() {
    let now = Utc::now();
    let past = now - Duration::hours(2) - Duration::minutes(32);
    let result = calculate_duration(past);
    assert!(result.contains("hr"), "Expected 'hr' in: {}", result);
    assert!(result.contains("min"), "Expected 'min' in: {}", result);
    assert!(
        !result.contains("sec"),
        "Should not include seconds: {}",
        result
    );
}

#[test]
fn test_hours_minutes_and_seconds_only_shows_two() {
    let now = Utc::now();
    let past = now - Duration::hours(1) - Duration::minutes(45) - Duration::seconds(30);
    let result = calculate_duration(past);
    assert!(result.contains("hr"), "Expected 'hr' in: {}", result);
    assert!(result.contains("min"), "Expected 'min' in: {}", result);
    assert!(
        !result.contains("sec"),
        "Should not include seconds when hours+minutes present: {}",
        result
    );
}

#[test]
fn test_zero_duration() {
    let now = Utc::now();
    let result = calculate_duration(now);
    assert_eq!(
        result, "",
        "Zero duration should return empty string: {}",
        result
    );
}
