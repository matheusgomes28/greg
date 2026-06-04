use anyhow::Context;
use chrono::{
    DateTime, Datelike, Local, Month, NaiveDate, TimeZone, Weekday
};
use std::convert::From;

// This model's purpose is meant solely for the view of
// a calendar. This means we're only concerned with what
// day of the week the month starts and how many days it
// has. We may eventually implement startt day in relation
// to a particular day of the week, so we can configure the
// view itself to start on any day of the week.
pub struct CalendarView {
    pub start_day: u8,
    pub n_days: u8,
    pub month_name: String,
}

impl CalendarView {

    pub fn new(month: u8, year: i32) -> anyhow::Result<CalendarView> {

        // TODO: There's probably a way to just use the date time here
        let dt = NaiveDate::from_ymd_opt(year, month as u32, 1)
            .context("could not create internal naive data")?
            .and_hms_opt(0, 0, 1)
            .context("could not add zero hours")?
            .and_local_timezone(Local)
            .earliest()
            .context("could not add timezone")?;


        Ok(CalendarView::from(dt))
    }
}

impl<T> From<DateTime<T>> for CalendarView where T: TimeZone  {
    fn from(datetime: DateTime<T>) -> Self {
        // Here we do a succ() because our calendar starts Sunday,
        // whereas the Weekday considers 0 to be Monday.
        let weekday = datetime.weekday().succ() as i32;

        let month_day = datetime.day0() as i32; // 1
        let target_day = (weekday - month_day).rem_euclid(7) as u8; // 

        // Safe: target day will always be 0-6
        // let start_day= Weekday::try_from(target_day).unwrap().num_days_from_monday() as u8;
        let start_day= Weekday::try_from(target_day).unwrap().num_days_from_monday() as u8;

        // TODO: figure out n days for this month
        let n_days = datetime.num_days_in_month();

        let month_name = Month::try_from(u8::try_from(datetime.month()).unwrap())
            .map(|a| a.name().to_string())
            .unwrap();

        CalendarView{
            start_day,
            n_days,
            month_name
        }
    }
    // TODO: possibly need another one just for date
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_starting_dates() -> anyhow::Result<()> {

        let test_cases = [
            ("2026 May 2 12:09:14.274 +0000", 5, 31, "May"),
            ("2026 Aug 15 12:09:14.274 +0000", 6, 31, "August"),
            ("2026 Sep 2 12:09:14.274 +0000", 2, 30, "September"),
        ];

        for (datetime_str, start_date, n_days, month_name) in test_cases {
            let desired_date = DateTime::parse_from_str(datetime_str, "%Y %b %d %H:%M:%S%.3f %z")?;
            let calendar = CalendarView::from(desired_date);

            assert_eq!(calendar.start_day, start_date);
            assert_eq!(calendar.n_days, n_days);
            assert_eq!(calendar.month_name, month_name);
        }

        Ok(())
    }

    #[test]
    fn correct_leap_year_feb_days() -> anyhow::Result<()> {
        let test_cases = [
            "1584 Feb 2 12:09:14.274 +0000",
            "1600 Feb 2 12:09:14.274 +0000",
            "2000 Feb 2 12:09:14.274 +0000",
            "2020 Feb 2 12:09:14.274 +0000",
            "2024 Feb 15 12:09:14.274 +0000",
            "2028 Feb 2 12:09:14.274 +0000",
            "2032 Feb 2 12:09:14.274 +0000",
            "2036 Feb 2 12:09:14.274 +0000",
            "2040 Feb 2 12:09:14.274 +0000",
            "2044 Feb 2 12:09:14.274 +0000",
        ];

        for datetime_str in test_cases {
            let desired_date = DateTime::parse_from_str(datetime_str, "%Y %b %d %H:%M:%S%.3f %z")?;
            let calendar = CalendarView::from(desired_date);

            assert_eq!(calendar.n_days, 29);
        }

        Ok(())
    }

    #[test]
    fn correct_starting_days_2026() -> anyhow::Result<()> {

        // test_case = ((month, year), (exp_start, exp_days, exp_month))
        let test_cases = [
            ((1, 2026), (4, 31, "January")),
            ((2, 2026), (0, 28, "February")),
            ((3, 2026), (0, 31, "March")),
            ((4, 2026), (3, 30, "April")),
            ((5, 2026), (5, 31, "May")),
            ((6, 2026), (1, 30, "June")),
            ((7, 2026), (3, 31, "July")),
            ((8, 2026), (6, 31, "August")),
            ((9, 2026), (2, 30, "September")),
            ((10, 2026), (4, 31, "October")),
            ((11, 2026), (0, 30, "November")),
            ((12, 2026), (2, 31, "December")),
        ];

        for ((input_month, input_year), (exp_start, exp_days, exp_month)) in test_cases {
            let calendar_view = CalendarView::new(input_month, input_year)?;

            assert_eq!(calendar_view.start_day, exp_start);
            assert_eq!(calendar_view.n_days, exp_days);
            assert_eq!(calendar_view.month_name, exp_month);
        }

        Ok(())
    }
}

