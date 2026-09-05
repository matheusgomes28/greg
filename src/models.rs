use anyhow::Context;
use chrono::{DateTime, Datelike, Duration, Local, Month, NaiveDate, TimeZone, Utc, Weekday};
use std::{collections::BTreeMap, convert::From};

#[derive(Clone, Debug, PartialEq)]
pub struct Event<T: TimeZone> {
    pub title: String,
    pub desc: String,
    pub start: DateTime<T>,
    pub end: DateTime<T>,
}

pub struct EventStore<T: TimeZone> {
    // TODO: What's the best way to store the
    // TODO: events given the public interface
    // TODO: for qurying them
    store: BTreeMap<DateTime<T>, Vec<Event<T>>>,
}

impl<T> EventStore<T>
where
    T: TimeZone,
{
    // This function should return events sorted
    // by the date tim e in chronological order
    // TODO: This should probably return an error if invalid range
    pub fn by_range(&self, low: DateTime<T>, high: DateTime<T>) -> Option<Vec<Event<T>>> {
        if low > high {
            return None;
        }

        let events = self
            .store
            .range(low..high)
            .flat_map(|(_, v)| v.iter().cloned())
            .collect::<Vec<Event<T>>>();

        (!events.is_empty()).then_some(events)
    }

    pub fn add(&mut self, dt: DateTime<T>, evt: Event<T>) {
        if let Some(events) = self.store.get_mut(&dt) {
            events.push(evt);
            return;
        }

        self.store.insert(dt, vec![evt]);
    }
}

impl<T> Default for EventStore<T>
where
    T: TimeZone,
{
    fn default() -> Self {
        Self {
            store: Default::default(),
        }
    }
}

impl EventStore<Utc> {
    // This function should return events sorted
    // by the date time in chronological order
    pub fn by_month(&self) -> Vec<Event<Utc>> {
        todo!()
    }

    // This function should return events sorted
    // by the date tim e in chronological order
    // UTC time only
    // TODO: This should probably return an error if invalid range
    pub fn by_day(&self, day: u8, month: u8, year: i32) -> Option<Vec<Event<Utc>>> {
        if let Some(low) = Utc
            .with_ymd_and_hms(year, month as u32, day as u32, 0, 0, 0)
            .single()
        {
            let high = low + Duration::days(1);
            return self.by_range(low, high);
        }

        None
    }
}

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

impl<T> From<DateTime<T>> for CalendarModel
where
    T: TimeZone,
{
    fn from(datetime: DateTime<T>) -> Self {
        // Here we do a succ() because our calendar starts Sunday,
        // whereas the Weekday considers 0 to be Monday.
        let weekday = datetime.weekday().succ() as i32;

        let today = datetime.day0() as i32; // 1
        let target_day = (weekday - today).rem_euclid(7) as u8; // 

        // Safe: target day will always be 0-6
        // let start_day= Weekday::try_from(target_day).unwrap().num_days_from_monday() as u8;
        let start_day = Weekday::try_from(target_day)
            .unwrap()
            .num_days_from_monday() as u8;

        // TODO: figure out n days for this month
        let n_days = datetime.num_days_in_month();

        let month_name = Month::try_from(u8::try_from(datetime.month()).unwrap())
            .map(|a| a.name().to_string())
            .unwrap();

        CalendarModel {
            start_day,
            n_days,
            today: Some(today as u8 + 1),
            // TODO: Can probably remove the month_name - don't need it
            month_name,
        }
    }
    // TODO: possibly need another one just for date
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    // Tests for the CalendarModel
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
    fn correct_calendars_2026(
        #[case] input_month: u8,
        #[case] input_year: i32,
        #[case] exp_start: u8,
        #[case] exp_days: u8,
        #[case] exp_month: &str,
    ) -> anyhow::Result<()> {
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

    // tests for the EventStore<Utc>
    #[rstest]
    #[case::recent_year(2000, 9000)]
    #[case::jesus_birth(0, 800000)]
    #[case::today(2026, 1)]
    fn nothing_in_range(#[case] year: i32, #[case] days: i64) {
        let empty_store: EventStore<Utc> = Default::default();
        let low = Utc.with_ymd_and_hms(year, 1, 1, 0, 0, 0).unwrap();
        let high = low + Duration::days(days);
        assert_eq!(empty_store.by_range(low, high), None);
    }

    #[rstest]
    #[case::reverse_range(Utc.with_ymd_and_hms(2026, 7, 30, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 7, 10, 0, 0, 0).unwrap())]
    #[case::equal_range(Utc.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap())]
    fn invalid_ranges(#[case] low: DateTime<Utc>, #[case] high: DateTime<Utc>) {
        let event_dt = Utc.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap();
        let store = EventStore::<Utc> {
            store: BTreeMap::from([(
                event_dt,
                vec![Event::<Utc> {
                    title: "test".to_string(),
                    desc: "test".to_string(),
                    start: event_dt,
                    end: event_dt,
                }],
            )]),
        };

        let result = store.by_range(low, high);
        assert_eq!(result, None);
    }

    #[rstest]
    #[case::day_range(Utc.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap(), Utc.with_ymd_and_hms(2026, 7, 21, 0, 0, 0).unwrap())]
    fn happy_path(#[case] low: DateTime<Utc>, #[case] high: DateTime<Utc>) {
        let event_dt = Utc.with_ymd_and_hms(2026, 7, 20, 0, 0, 0).unwrap();
        let store = EventStore::<Utc> {
            store: BTreeMap::from([(
                event_dt,
                vec![Event::<Utc> {
                    title: "test title".to_string(),
                    desc: "test desc".to_string(),
                    start: event_dt,
                    end: event_dt,
                }],
            )]),
        };

        let result = store.by_range(low, high);
        assert!(result.is_some());

        let values = result.unwrap();
        assert_eq!(values.len(), 1);

        let value = values[0].clone();
        assert_eq!(value.title, "test title");
        assert_eq!(value.desc, "test desc");
        assert_eq!(value.start, event_dt);
    }
}
