use time::Date;

use super::models::{Frequency, RecurringDebt};

/// Calculate the next occurrence date based on frequency
pub fn calculate_next_occurrence(current_date: Date, frequency: &Frequency) -> Date {
    match frequency {
        Frequency::Daily => current_date.saturating_add(time::Duration::days(1)),
        Frequency::Weekly => current_date.saturating_add(time::Duration::weeks(1)),
        Frequency::Monthly => {
            // Add one month - handle edge cases like Jan 31 -> Feb 28
            let year = current_date.year();
            let month = current_date.month();
            let day = current_date.day();

            let (new_year, new_month) = if month == time::Month::December {
                (year + 1, time::Month::January)
            } else {
                (year, month.next())
            };

            // Handle day overflow (e.g., Jan 31 -> Feb 28/29)
            let max_day = new_month.length(new_year);
            let new_day = day.min(max_day);

            Date::from_calendar_date(new_year, new_month, new_day).unwrap_or(current_date)
        }
        Frequency::Yearly => {
            let year = current_date.year();
            let month = current_date.month();
            let day = current_date.day();

            // Handle leap year edge case (Feb 29 -> Feb 28 in non-leap year)
            let new_year = year + 1;
            let max_day = month.length(new_year);
            let new_day = day.min(max_day);

            Date::from_calendar_date(new_year, month, new_day).unwrap_or(current_date)
        }
    }
}

/// Check if a recurring debt should generate a new shared debt
pub fn should_generate(recurring_debt: &RecurringDebt, today: Date) -> bool {
    // Not active
    if !recurring_debt.is_active {
        return false;
    }

    // Past end date
    if let Some(end_date) = recurring_debt.end_date {
        if today > end_date {
            return false;
        }
    }

    // Not yet time to generate
    if today < recurring_debt.next_generation_date {
        return false;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    #[test]
    fn test_calculate_next_occurrence_daily() {
        let date = Date::from_calendar_date(2026, Month::February, 15).unwrap();
        let next = calculate_next_occurrence(date, &Frequency::Daily);
        assert_eq!(
            next,
            Date::from_calendar_date(2026, Month::February, 16).unwrap()
        );
    }

    #[test]
    fn test_calculate_next_occurrence_weekly() {
        let date = Date::from_calendar_date(2026, Month::February, 15).unwrap();
        let next = calculate_next_occurrence(date, &Frequency::Weekly);
        assert_eq!(
            next,
            Date::from_calendar_date(2026, Month::February, 22).unwrap()
        );
    }

    #[test]
    fn test_calculate_next_occurrence_monthly() {
        let date = Date::from_calendar_date(2026, Month::January, 31).unwrap();
        let next = calculate_next_occurrence(date, &Frequency::Monthly);
        // Jan 31 -> Feb 28 (2026 is not a leap year)
        assert_eq!(
            next,
            Date::from_calendar_date(2026, Month::February, 28).unwrap()
        );
    }

    #[test]
    fn test_calculate_next_occurrence_yearly() {
        let date = Date::from_calendar_date(2024, Month::February, 29).unwrap(); // Leap year
        let next = calculate_next_occurrence(date, &Frequency::Yearly);
        // Feb 29 2024 -> Feb 28 2025 (not a leap year)
        assert_eq!(
            next,
            Date::from_calendar_date(2025, Month::February, 28).unwrap()
        );
    }
}
