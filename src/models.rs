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
#[derive(Debug, Clone)]
pub struct CalendarModel {
    pub start_day: u8,
    pub n_days: u8,
    pub today: Option<u8>,
    pub month_name: String,
}

impl CalendarModel {

    pub fn new(day: Option<u8>, month: u8, year: i32) -> anyhow::Result<CalendarModel> {
        if year < 0 {
            anyhow::bail!("calendar cannot receive a negative number");
        }

        // TODO: There's probably a way to just use the date time here
        let dt = NaiveDate::from_ymd_opt(year, month as u32, day.unwrap_or(1) as u32)
            .context("could not create internal naive data")?
            .and_hms_opt(0, 0, 1)
            .context("could not add zero hours")?
            .and_local_timezone(Local)
            .earliest()
            .context("could not add timezone")?;


        Ok(CalendarModel::from(dt))
    }
}

impl Default for CalendarModel {

    fn default() -> Self {
        let time_now = Local::now();
        CalendarModel::new(Some(time_now.day() as u8), time_now.month() as u8, time_now.year()).unwrap()
    }
}

impl<T> From<DateTime<T>> for CalendarModel where T: TimeZone  {
    fn from(datetime: DateTime<T>) -> Self {
        // Here we do a succ() because our calendar starts Sunday,
        // whereas the Weekday considers 0 to be Monday.
        let weekday = datetime.weekday().succ() as i32;

        let today = datetime.day0() as i32; // 1
        let target_day = (weekday - today).rem_euclid(7) as u8; // 

        // Safe: target day will always be 0-6
        // let start_day= Weekday::try_from(target_day).unwrap().num_days_from_monday() as u8;
        let start_day= Weekday::try_from(target_day).unwrap().num_days_from_monday() as u8;

        // TODO: figure out n days for this month
        let n_days = datetime.num_days_in_month();

        let month_name = Month::try_from(u8::try_from(datetime.month()).unwrap())
            .map(|a| a.name().to_string())
            .unwrap();

        CalendarModel{
            start_day,
            n_days,
            today: Some(today as u8 + 1),
            // TODO: Can probably remove the month_name - don't need it
            month_name
        }
    }
    // TODO: possibly need another one just for date
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::year_1584("1584 Feb 2 12:09:14.274 +0000")]
    #[case::year_1600("1600 Feb 2 12:09:14.274 +0000")]
    #[case::year_2000("2000 Feb 2 12:09:14.274 +0000")]
    #[case::year_2020("2020 Feb 2 12:09:14.274 +0000")]
    #[case::year_2024("2024 Feb 15 12:09:14.274 +0000")]
    #[case::year_2028("2028 Feb 2 12:09:14.274 +0000")]
    #[case::year_2032("2032 Feb 2 12:09:14.274 +0000")]
    #[case::year_2036("2036 Feb 2 12:09:14.274 +0000")]
    #[case::year_2040("2040 Feb 2 12:09:14.274 +0000")]
    #[case::year_2044("2044 Feb 2 12:09:14.274 +0000")]
    fn correct_leap_year_feb_days(#[case] dt_str: &str) -> anyhow::Result<()> {
        let desired_date = DateTime::parse_from_str(dt_str, "%Y %b %d %H:%M:%S%.3f %z")?;
        let calendar = CalendarModel::from(desired_date);

        assert_eq!(calendar.n_days, 29);

        Ok(())
    }

    #[rstest]
    #[case::january(1, 2026, 4, 31, "January")]
    #[case::february(2, 2026, 0, 28, "February")]
    #[case::march(3, 2026, 0, 31, "March")]
    #[case::april(4, 2026, 3, 30, "April")]
    #[case::may(5, 2026, 5, 31, "May")]
    #[case::june(6, 2026, 1, 30, "June")]
    #[case::july(7, 2026, 3, 31, "July")]
    #[case::august(8, 2026, 6, 31, "August")]
    #[case::september(9, 2026, 2, 30, "September")]
    #[case::october(10, 2026, 4, 31, "October")]
    #[case::november(11, 2026, 0, 30, "November")]
    #[case::december(12, 2026, 2, 31, "December")]
    fn correct_calendars_2026(#[case] input_month: u8, #[case] input_year: i32, #[case] exp_start: u8, #[case] exp_days: u8, #[case] exp_month: &str) -> anyhow::Result<()> {
        let calendar_view = CalendarModel::new(None, input_month, input_year)?;

        assert_eq!(calendar_view.start_day, exp_start);
        assert_eq!(calendar_view.n_days, exp_days);
        assert_eq!(calendar_view.month_name, exp_month);

        Ok(())
    }

    #[rstest]
    #[case::zero_month(0, 1996)]
    #[case::big_month(13, 1996)]
    #[case::negative_year(2, -1)]
    #[case::both_incorrect(15, -1)]
    fn incorrect_date(#[case] month: u8, #[case] year: i32) {
        let maybe_calendar = CalendarModel::new(None, month, year);
        assert!(maybe_calendar.is_err());
    }
}

