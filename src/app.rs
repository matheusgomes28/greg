use std::{
    fs::File, io::{self, BufReader}
};

use crate::{
    ics_utils::read_events,
    models::{CalendarModel, EventStore},
    views::{CalendarView, DayColor, DayStyle},
};

use anyhow::Context;
use chrono::{Datelike, Duration, Local, NaiveDate, TimeZone};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

use ratatui::{DefaultTerminal, Frame};

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    View,
    Select,
}

impl From<&AppMode> for String {
    fn from(value: &AppMode) -> Self {
        match value {
            AppMode::View => "View".into(),
            AppMode::Select => "Select".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct App {
    exit: bool,

    current_month: u8,
    current_year: i32,

    calendar_model: CalendarModel,
    calendar_view: CalendarView,

    // Additions for highlighing today
    this_day: u8,
    this_month: u8,
    this_year: i32,
    selected_day: u8,
    mode: AppMode,
    events: EventStore<Local>,

    // TODO: Need to read all of these with the ICS reader
    // TODO: library. Should read all of the events in all
    // TODO: ics files and then store them as a calendar model
    // TODO: where we can request events per month and day
    // Additions for reading ICS files
    pub ics_directory: Option<String>,
}

const TODAY_STYLE: DayStyle = DayStyle::Colored(DayColor::Blue);
const SELECTED_STYLE: DayStyle = DayStyle::Highlighted(DayColor::Blue);
const EVENT_STYLE: DayStyle = DayStyle::Colored(DayColor::Green);

impl Default for App {
    fn default() -> Self {
        let time_now = Local::now();

        let calendar_model = CalendarModel::from(time_now);

        let this_day = time_now.day() as u8;
        let this_month = time_now.month() as u8;
        let this_year = time_now.year();
        let mode = AppMode::View;

        App {
            exit: false,
            current_month: this_month,
            current_year: this_year,
            calendar_model: calendar_model.clone(),
            calendar_view: CalendarView {
                title: String::from(&mode),
                month_name: calendar_model.month_name,
                start_day: calendar_model.start_day,
                year: time_now.year(),
                n_days: calendar_model.n_days as usize,
                styled_days: vec![(time_now.day() as u8, TODAY_STYLE)],
                event: None,
            },

            this_day,
            this_month,
            this_year,
            selected_day: this_day,
            mode,
            events: Default::default(),
            ics_directory: None,
        }
    }
}

impl App {
    // TODO: I don't like this, but it looks rustonic...
    pub fn with_ics_dir(&self, ics_directory: Option<String>) -> anyhow::Result<Self> {
        let mut ret = self.clone();
        ret.ics_directory = ics_directory.clone();
        ret.events = events_from_file(&ics_directory)?;

        let new_styles = get_styles(
            &ics_directory,
            self.this_month,
            self.this_year,
            self.calendar_model.n_days,
        )?;
        ret.calendar_view.styled_days =
            [new_styles, self.calendar_view.styled_days.clone()].concat();

        Ok(ret)
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        // This is where we need the view!
        frame.render_widget(&self.calendar_view, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.}
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match self.mode {
                    AppMode::View => self.handle_key_event_view(key_event),
                    AppMode::Select => self.handle_key_event_select(key_event),
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event_view(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Up | KeyCode::Char('j') => self.next_calendar(),
            KeyCode::Down | KeyCode::Char('k') => self.prev_calendar(),
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Enter => self.switch_to_select_mode(),
            _ => {}
        }
        self.calendar_view.title = String::from(&self.mode);
    }

    fn handle_key_event_select(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Down | KeyCode::Char('j') => self.selection_down(),
            KeyCode::Up | KeyCode::Char('k') => self.selection_up(),
            KeyCode::Left | KeyCode::Char('h') => self.selection_left(),
            KeyCode::Right | KeyCode::Char('l') => self.selection_right(),
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            KeyCode::Enter => self.switch_to_view_mode(),
            _ => {}
        }
        self.calendar_view.title = String::from(&self.mode);
    }

    fn switch_to_select_mode(&mut self) {
        self.mode = AppMode::Select;

        // TODO: This should contingent on today being in this month / year
        self.selected_day = self.this_day;

        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        styled_days.push((self.selected_day, SELECTED_STYLE));

        self.calendar_view = CalendarView {
            title: String::from("Select"),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event: None,
        };
    }

    fn switch_to_view_mode(&mut self) {
        self.mode = AppMode::View;

        // TODO: This should contingent on today being in this month / year
        self.selected_day = self.this_day;

        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        if let Some(maybe_today) = NaiveDate::from_ymd_opt(
            self.current_year,
            self.current_month as u32,
            self.this_day as u32,
        ) && self.is_today(maybe_today)
        {
            styled_days.push((self.this_day, TODAY_STYLE));
        }

        self.calendar_view = CalendarView {
            title: String::from("View"),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event: None,
        };
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn is_today(&self, target_time: NaiveDate) -> bool {
        let target_day = target_time.day() as u8;
        let target_month = target_time.month() as u8;
        let target_year = target_time.year();

        target_year == self.this_year
            && target_month == self.this_month
            && target_day == self.this_day
    }

    // Calendar Select functions
    fn selection_down(&mut self) {
        let day_offset = 7;
        let next_selection = self.selected_day + day_offset;
        self.selected_day = std::cmp::min(next_selection, self.calendar_model.n_days);

        // TODO: Copied everywhere
        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        styled_days.push((self.selected_day, SELECTED_STYLE));

        // TODO: This only supports one event!!!
        let low = Local.with_ymd_and_hms(self.current_year, self.current_month as u32, self.selected_day as u32, 0, 0, 0).unwrap();
        let high = low + Duration::days(1);
        let event = self.events
            .by_range(low, high)
            .and_then(|f| f.first().cloned());

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event,
        };
    }

    fn selection_right(&mut self) {
        let day_offset = 1;
        let next_selection = self.selected_day + day_offset;
        self.selected_day = std::cmp::min(next_selection, self.calendar_model.n_days);

        // TODO: Copied everywhere
        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        styled_days.push((self.selected_day, SELECTED_STYLE));

        // TODO: This only supports one event!!!
        let low = Local.with_ymd_and_hms(self.current_year, self.current_month as u32, self.selected_day as u32, 0, 0, 0).unwrap();
        let high = low + Duration::days(1);
        let event = self.events
            .by_range(low, high)
            .and_then(|f| f.first().cloned());

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event,
        };
    }

    fn selection_up(&mut self) {
        let day_offset: i16 = -7;
        let next_selection = self.selected_day as i16 + day_offset;
        self.selected_day = std::cmp::max(next_selection, 1) as u8;

        // TODO: Copied everywhere
        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        styled_days.push((self.selected_day, SELECTED_STYLE));

        // TODO: This only supports one event!!!
        let low = Local.with_ymd_and_hms(self.current_year, self.current_month as u32, self.selected_day as u32, 0, 0, 0).unwrap();
        let high = low + Duration::days(1);
        let event = self.events
            .by_range(low, high)
            .and_then(|f| f.first().cloned());

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event,
        };
    }

    fn selection_left(&mut self) {
        let day_offset: i16 = -1;
        let next_selection = self.selected_day as i16 + day_offset;
        self.selected_day = std::cmp::max(next_selection, 1) as u8;

        // TODO: Copied everywhere
        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        styled_days.push((self.selected_day, SELECTED_STYLE));

        // TODO: This only supports one event!!!
        let low = Local.with_ymd_and_hms(self.current_year, self.current_month as u32, self.selected_day as u32, 0, 0, 0).unwrap();
        let high = low + Duration::days(1);
        let event = self.events
            .by_range(low, high)
            .and_then(|f| f.first().cloned());

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event,
        };
    }

    // Calendar View functions
    fn next_calendar(&mut self) {
        self.current_year += self.current_month as i32 / 12;
        self.current_month = self.current_month.rem_euclid(12) + 1;

        self.calendar_model =
            CalendarModel::new(None, self.current_month, self.current_year).unwrap();

        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        if let Some(maybe_today) = NaiveDate::from_ymd_opt(
            self.current_year,
            self.current_month as u32,
            self.this_day as u32,
        ) && self.is_today(maybe_today)
        {
            styled_days.push((self.this_day, TODAY_STYLE));
        }

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event: None,
        };
    }

    fn prev_calendar(&mut self) {
        self.current_year -= (12 - (self.current_month as i32 - 1).rem_euclid(12)) / 12;
        self.current_month = if self.current_month == 1 {
            12
        } else {
            self.current_month - 1
        };

        // TODO: ideally we would update instead of recreate?
        self.calendar_model =
            CalendarModel::new(None, self.current_month, self.current_year).unwrap();

        let mut styled_days = get_styles(
            &self.ics_directory,
            self.current_month,
            self.current_year,
            self.calendar_model.n_days,
        )
        .unwrap_or(vec![]);

        if let Some(maybe_today) = NaiveDate::from_ymd_opt(
            self.current_year,
            self.current_month as u32,
            self.this_day as u32,
        ) && self.is_today(maybe_today)
        {
            styled_days.push((self.this_day, TODAY_STYLE));
        }

        self.calendar_view = CalendarView {
            title: self.calendar_view.title.clone(),
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            styled_days,
            event: None,
        };
    }
}

fn events_from_file(ics_directory: &Option<String>) -> anyhow::Result<EventStore<Local>> {
    let file_path = ics_directory.clone().context("file path was not given")?;

    let file = File::open(file_path)?;
    let file_buf = BufReader::new(file);

    // TODO: We want to save this inside the App as
    // TODO: the state of the events, not read these files
    // TODO: whenever
    read_events(file_buf, &Local)
}

// TODO: This should probably be per calendar config
fn get_styles(
    ics_directory: &Option<String>,
    month: u8,
    year: i32,
    n_days: u8,
) -> anyhow::Result<Vec<(u8, DayStyle)>> {
    let events = events_from_file(ics_directory)?;

    let low = Local
        .with_ymd_and_hms(year, month as u32, 1, 0, 0, 0)
        .single()
        .context("error creating the range value")?;
    let high = low + Duration::days(n_days as i64);

    let this_months_events = events.by_range(low, high);

    if let Some(this_months_events) = this_months_events {
        let mut res = Vec::<(u8, DayStyle)>::new();
        for event in this_months_events {
            let day = event.start.day() as u8;
            res.push((day, EVENT_STYLE));
        }
        return Ok(res);
    }

    Ok(vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::jan_2026(false, 1, 2026, 2, 2026)]
    #[case::dec_2026(false, 12, 2026, 1, 2027)]
    #[case::dec_1999(false, 12, 1999, 1, 2000)]
    fn next_calendar(
        #[case] exit: bool,
        #[case] current_month: u8,
        #[case] current_year: i32,
        #[case] exp_month: u8,
        #[case] exp_year: i32,
    ) -> anyhow::Result<()> {
        let calendar_model = CalendarModel::new(None, current_month, current_year)?;
        let calendar_view = CalendarView {
            title: String::from("View"),
            month_name: calendar_model.month_name.clone(),
            start_day: calendar_model.start_day,
            year: current_year,
            n_days: calendar_model.n_days as usize,
            styled_days: Default::default(),
            event: None,
        };

        let mut app = App {
            exit,
            current_month,
            current_year,
            calendar_model,
            calendar_view,
            ..Default::default()
        };

        app.next_calendar();

        assert_eq!(exp_month, app.current_month);
        assert_eq!(exp_year, app.current_year);

        Ok(())
    }

    #[rstest]
    #[case::jan_2026(false, 1, 2026, 12, 2025)]
    #[case::dec_2026(false, 12, 2026, 11, 2026)]
    fn prev_calendar(
        #[case] exit: bool,
        #[case] current_month: u8,
        #[case] current_year: i32,
        #[case] exp_month: u8,
        #[case] exp_year: i32,
    ) -> anyhow::Result<()> {
        let calendar_model = CalendarModel::new(None, current_month, current_year)?;
        let calendar_view = CalendarView {
            title: String::from("View"),
            month_name: calendar_model.month_name.clone(),
            start_day: calendar_model.start_day,
            year: current_year,
            n_days: calendar_model.n_days as usize,
            styled_days: Default::default(),
            event: None,
        };
        let mut app = App {
            exit,
            current_month,
            current_year,
            calendar_model,
            calendar_view,
            ..Default::default()
        };

        app.prev_calendar();

        assert_eq!(exp_month, app.current_month);
        assert_eq!(exp_year, app.current_year);
        Ok(())
    }
}
