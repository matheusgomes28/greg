
use crate::io;

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
    counter: u8,
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
            KeyCode::Left => self.decrement_counter(),
            KeyCode::Right => self.increment_counter(),
            _ => {}
        }
    }

    fn decrement_counter(&mut self) {
        self.counter -= 1;
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    pub fn render_weeks_header(&self, row: &[Rect], buf: &mut Buffer) {
        let header_style = Style::default()
            .bold()
            .fg(Color::Blue);

        for (day, &cell) in ["S", "S", "M", "T", "W", "T", "F"].into_iter().zip(row) {
            Span::styled(day, header_style).render(cell, buf);
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {

        let block = Block::bordered()
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .title(Line::from("Matheus' Calendar"));

        let col_constraints = (0..7).map(|_| Constraint::Length(4));
        let row_constraits = (0..6).map(|_| Constraint::Length(1));

        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraits).spacing(0);

        let inner_area = block
            .inner(area)
            .centered_horizontally(Constraint::Length(7 * 4))
            .centered_vertically(Constraint::Length(6 * 1));

        let rows = vertical.split(inner_area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        let header_cells = cells.clone().take(7).collect::<Vec<_>>();
        self.render_weeks_header(&header_cells, buf);

        // TODO: Figure out whether this month has 28,30,31 days
        let month_days = 28;

        for (i, cell) in cells.skip(7).take(month_days).enumerate() {
            let text = Text::styled(format!("{}", i + 1), Style::default().red());
            Paragraph::new(text)
                .render(cell, buf);
        }

        block.render(area, buf);
    }
}

