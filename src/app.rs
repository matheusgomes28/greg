
use crate::{io, models::CalendarView};

use chrono::DateTime;
use crossterm::event::{
    self, Event, KeyCode, KeyEvent, KeyEventKind
};

use ratatui::{
    DefaultTerminal,
    Frame,
    buffer::Buffer,
    layout::{
        Constraint,
        Layout,
        Rect
    },
    style::{Color, Style},
    text::{
        Line, Span, Text
    },
    widgets::{
        Block,
        Paragraph,
        Widget
    }
};

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
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
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn render_weeks_header(&self, row: &[Rect], buf: &mut Buffer) {
        let header_style = Style::default()
            .bold()
            .fg(Color::Blue);

        for (day, &cell) in ["S", "M", "T", "W", "T", "F", "S"].into_iter().zip(row) {
            Span::styled(day, header_style).render(cell, buf);
        }
    }

    pub fn render_days(&self, cells: &[Rect], buf: &mut Buffer, start_day: i32, n_days: usize) {
        for (i, &cell) in cells.iter().take(n_days + start_day as usize).enumerate() {
            // Offset the value of each cell by the starting day,
            // and only draw cells that have a value of > 1
            let month_day = (i as i32) - start_day + 1;
            if month_day <= 0 {
                continue;
            }

            let text = Text::styled(format!("{}", month_day), Style::default().red());
            Paragraph::new(text)
                .render(cell, buf);
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let datetime = DateTime::parse_from_str("2026 Sep 2 12:09:14.274 +0000", "%Y %b %d %H:%M:%S%.3f %z").unwrap();
        let calendar = CalendarView::from(datetime);

        let block = Block::bordered()
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .title(Line::from(calendar.month_name));

        let col_constraints = (0..7).map(|_| Constraint::Length(4));
        let row_constraits = (0..7).map(|_| Constraint::Length(1));

        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraits).spacing(0);

        let inner_area = block
            .inner(area)
            .centered_horizontally(Constraint::Length(7 * 4))
            .centered_vertically(Constraint::Length(7 * 1));

        let rows = vertical.split(inner_area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        let header_cells = cells.clone().take(7).collect::<Vec<_>>();
        let weeks_cells = cells.clone().skip(7).collect::<Vec<_>>();

        self.render_weeks_header(&header_cells, buf);
        self.render_days(&weeks_cells, buf, calendar.start_day as i32, calendar.n_days as usize);

        block.render(area, buf);
    }
}

