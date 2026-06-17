
use crate::{io, models::CalendarModel, views::CalendarView};

use chrono::{Datelike, Local};
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind
};

use ratatui::{
    DefaultTerminal,
    Frame
};

#[derive(Debug)]
pub struct App {
    exit: bool,

    current_month: u8,
    current_year: i32,

    calendar_model: CalendarModel,
    calendar_view: CalendarView,
}

impl Default for App {

    fn default() -> Self {
        let time_now = Local::now();

        let calendar_model = CalendarModel::from(time_now);

        App{
            exit: false,
            current_month: time_now.month() as u8,
            current_year: time_now.year(),
            calendar_model: calendar_model.clone(),
            calendar_view: CalendarView{
                month_name: calendar_model.month_name,
                start_day: calendar_model.start_day,
                year: time_now.year(),
                n_days: calendar_model.n_days as usize,
                current_day: Some(time_now.day() as u8)
            }
        }
    }
}

impl App {
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
                self.handle_key_event(key_event)
            },
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Right | KeyCode::Char('j') => self.next_calendar(),
            KeyCode::Left | KeyCode::Char('k') => self.prev_calendar(),
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn next_calendar(&mut self) {

        self.current_year += self.current_month as i32 / 12;
        self.current_month = self.current_month.rem_euclid(12) + 1;

        // TODO: ideally we would update instead of recreate?
        self.calendar_model = CalendarModel::new(None, self.current_month, self.current_year).unwrap();

        self.calendar_view = CalendarView{
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            current_day: None
        };
    }

    fn prev_calendar(&mut self) {
        // TODO: Need to implement this
        self.current_year -= (12 - (self.current_month as i32 - 1).rem_euclid(12)) / 12;
        self.current_month = if self.current_month == 1 { 12 } else { self.current_month - 1 } ;

        // TODO: ideally we would update instead of recreate?
        self.calendar_model = CalendarModel::new(None, self.current_month, self.current_year).unwrap();

        self.calendar_view = CalendarView{
            month_name: self.calendar_model.month_name.clone(),
            start_day: self.calendar_model.start_day,
            year: self.current_year,
            n_days: self.calendar_model.n_days as usize,
            current_day: None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;


    #[rstest]
    #[case::jan_2026(false, 1, 2026, None, 2, 2026)]
    #[case::dec_2026(false, 12, 2026, None, 1, 2027)]
    #[case::dec_1999(false, 12, 1999, None, 1, 2000)]
    fn next_calendar(#[case] exit: bool, #[case] current_month: u8, #[case] current_year: i32, #[case] current_day: Option<u8>, #[case] exp_month: u8, #[case] exp_year: i32) -> anyhow::Result<()> {

        let calendar_model = CalendarModel::new(None, current_month, current_year)?;
        let calendar_view = CalendarView{
            month_name: calendar_model.month_name.clone(),
            start_day: calendar_model.start_day,
            year: current_year,
            n_days: calendar_model.n_days as usize,
            current_day: None
        };

        let mut app = App{
            exit,
            current_month,
            current_year,
            calendar_model,
            calendar_view,
        };

        app.next_calendar();

        assert_eq!(exp_month, app.current_month);
        assert_eq!(exp_year, app.current_year);

        Ok(())
    }

    #[rstest]
    #[case::jan_2026(false, 1, 2026, None, 12, 2025)]
    #[case::dec_2026(false, 12, 2026, None, 11, 2026)]
    fn prev_calendar(#[case] exit: bool, #[case] current_month: u8, #[case] current_year: i32, #[case] current_day: Option<u8>, #[case] exp_month: u8, #[case] exp_year: i32) -> anyhow::Result<()> {
        let calendar_model = CalendarModel::new(None, current_month, current_year)?;
        let calendar_view = CalendarView{
            month_name: calendar_model.month_name.clone(),
            start_day: calendar_model.start_day,
            year: current_year,
            n_days: calendar_model.n_days as usize,
            current_day: None
        };
        let mut app = App{
            exit,
            current_month,
            current_year,
            calendar_model,
            calendar_view,
        };

        app.prev_calendar();

        assert_eq!(exp_month, app.current_month);
        assert_eq!(exp_year, app.current_year);
        Ok(())
    }
}
