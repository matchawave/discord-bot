use chrono::Datelike;
use utils::ResponseError;

pub fn check_date_formats(date_str: &str) -> Result<(u32, u32, Option<i32>), ResponseError> {
    let dash_date: Vec<&str> = date_str.split("-").collect();
    parse_date_function(&dash_date)
        .or_else(|_| {
            let dot_date: Vec<&str> = date_str.split(".").collect();
            parse_date_function(&dot_date)
        })
        .or_else(|_| {
            let slash_date: Vec<&str> = date_str.split("/").collect();
            parse_date_function(&slash_date)
        })
}

fn parse_date_function(vector: &[&str]) -> Result<(u32, u32, Option<i32>), ResponseError> {
    let month: u32;
    let day: u32;
    let mut year = None;
    if vector.len() == 2 {
        month = (vector[0].parse::<u32>()).map_err(|_| "Invalid month format")?;
        day = (vector[1].parse::<u32>()).map_err(|_| "Invalid day format")?;
    } else if vector.len() == 3 {
        let first = vector[0];
        let last = vector[2];
        month = vector[1]
            .parse::<u32>()
            .map_err(|_| "Invalid month format")?;

        if first.len() == 4 {
            year = Some(first.parse::<i32>().map_err(|_| "Invalid year format")?);
            day = last.parse::<u32>().map_err(|_| "Invalid day format")?;
        } else if first.len() == 2 {
            day = first.parse::<u32>().map_err(|_| "Invalid day format")?;
            year = Some(last.parse::<i32>().map_err(|_| "Invalid year format")?);
        } else {
            return Err("Invalid date format".into());
        }
    } else {
        return Err("Invalid date format".into());
    }

    if day == 0 || day > 31 {
        return Err("Day must be between 1 and 31".into());
    }

    if month == 0 || month > 12 {
        return Err("Month must be between 1 and 12".into());
    }

    if let Some(year) = year {
        let now = chrono::Utc::now();
        let current_year = now.year();
        if year < 1900 || year > current_year {
            return Err(format!("Year must be between 1900 and {}", current_year).into());
        }

        let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
            .ok_or_else(|| "Invalid date".to_string())?; // This will return None if the date is invalid (e.g., February 30th on a non-leap year)

        // Check if the date is in the future
        let today = now.date_naive();
        if date > today {
            return Err("You cannot be born in the future!".into());
        }
    }
    Ok((month, day, year))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_month_day_only() {
        let result = parse_date_function(&["03", "15"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 3);
        assert_eq!(day, 15);
        assert_eq!(year, None);
    }

    #[test]
    fn test_valid_year_month_day() {
        let result = parse_date_function(&["1995", "03", "15"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 3);
        assert_eq!(day, 15);
        assert_eq!(year, Some(1995));
    }

    #[test]
    fn test_valid_day_month_year() {
        let result = parse_date_function(&["15", "03", "1995"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 3);
        assert_eq!(day, 15);
        assert_eq!(year, Some(1995));
    }

    #[test]
    fn test_invalid_empty_vector() {
        let result = parse_date_function(&[]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid date format")
        );
    }

    #[test]
    fn test_invalid_single_element() {
        let result = parse_date_function(&["15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid date format")
        );
    }

    #[test]
    fn test_invalid_four_elements() {
        let result = parse_date_function(&["15", "03", "1995", "extra"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid date format")
        );
    }

    #[test]
    fn test_invalid_month_zero() {
        let result = parse_date_function(&["00", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Month must be between 1 and 12")
        );
    }

    #[test]
    fn test_invalid_month_thirteen() {
        let result = parse_date_function(&["13", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Month must be between 1 and 12")
        );
    }

    #[test]
    fn test_invalid_day_zero() {
        let result = parse_date_function(&["03", "00"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Day must be between 1 and 31")
        );
    }

    #[test]
    fn test_invalid_day_thirty_two() {
        let result = parse_date_function(&["03", "32"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Day must be between 1 and 31")
        );
    }

    #[test]
    fn test_invalid_year_too_old() {
        let result = parse_date_function(&["1800", "03", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Year must be between 1900 and")
        );
    }

    #[test]
    fn test_invalid_year_future() {
        let result = parse_date_function(&["2030", "03", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Year must be between 1900 and")
        );
    }

    #[test]
    fn test_invalid_date_february_30() {
        let result = parse_date_function(&["2020", "02", "30"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date"));
    }

    #[test]
    fn test_invalid_date_february_29_non_leap_year() {
        let result = parse_date_function(&["2021", "02", "29"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid date"));
    }

    #[test]
    fn test_valid_date_february_29_leap_year() {
        let result = parse_date_function(&["2020", "02", "29"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 2);
        assert_eq!(day, 29);
        assert_eq!(year, Some(2020));
    }

    #[test]
    fn test_invalid_non_numeric_month() {
        let result = parse_date_function(&["ab", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid month format")
        );
    }

    #[test]
    fn test_invalid_non_numeric_day() {
        let result = parse_date_function(&["03", "xy"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid day format")
        );
    }

    #[test]
    fn test_invalid_non_numeric_year() {
        let result = parse_date_function(&["abcd", "03", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid year format")
        );
    }

    #[test]
    fn test_invalid_three_digit_first_element() {
        let result = parse_date_function(&["123", "03", "15"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid date format")
        );
    }

    #[test]
    fn test_future_date_with_year() {
        let result = parse_date_function(&["2026", "12", "25"]);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("You cannot be born in the future!")
        );
    }

    #[test]
    fn test_boundary_year_1900() {
        let result = parse_date_function(&["1900", "01", "01"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(year, Some(1900));
    }

    #[test]
    fn test_boundary_current_year() {
        let now = chrono::Utc::now();
        let current_year = now.year();
        let result = parse_date_function(&[&current_year.to_string(), "01", "01"]);
        assert!(result.is_ok());
        let (month, day, year) = result.unwrap();
        assert_eq!(month, 1);
        assert_eq!(day, 1);
        assert_eq!(year, Some(current_year));
    }
}
