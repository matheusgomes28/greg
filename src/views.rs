use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Color, Style, Stylize}, text::{Line, Span, Text}, widgets::{Block, Paragraph, Widget, Wrap},
};

use crate::models::Event;

const MIN_CALENDAR_WIDTH: u16 = 32;
const MIN_CALENDAR_HEIGHT: u16 = 13;

#[derive(Debug, Clone)]
pub enum DayColor {
    Red,
    Green,
    Blue,
}

impl Into<Color> for DayColor {
    fn into(self) -> Color {
        match self {
            DayColor::Red => Color::Red,
            DayColor::Green => Color::Green,
            DayColor::Blue => Color::Blue,
        }
    }
}

#[derive(Debug, Clone)]
pub enum DayStyle {
    Colored(DayColor),
    Bold(DayColor),
    Highlighted(DayColor),
    Normal,
}

impl Into<Style> for DayStyle {
    fn into(self) -> Style {
        match self {
            DayStyle::Normal => Style::new(),
            DayStyle::Bold(color) => Style::default().bold().fg(color.into()),
            DayStyle::Highlighted(color) => Style::default().bg(color.into()),
            DayStyle::Colored(color) => Style::default().fg(color.into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CalendarView {
    pub title: String,
    pub month_name: String,
    pub start_day: u8,
    pub year: i32,
    pub n_days: usize,
    pub styled_days: Vec<(u8, DayStyle)>,
    pub event: Option<Event<Local>>
}

impl Widget for &CalendarView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if (area.width < MIN_CALENDAR_WIDTH) || (area.height < MIN_CALENDAR_HEIGHT) {
            self.render_size_warning(area, buf);
            return;
        }

        let title_text = format!("{} {} - {}", self.year, self.month_name, self.title);
        let block = Block::bordered()
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .title(Line::from(title_text));

        // Constraints set the physical width and height for the
        // inner element
        // Note: the lengths here should be the total length of items
        // Note: inside.
        let horizontal_size = 7*4;


        // TODO: Figure out whether a fixes size is better for
        // TODO: when we have events showing up
        // let vertical_size: u16 = self.event
        //     .clone()
        //     .map_or(7, |_x| 10);
        let vertical_size: u16 = 10;

        let inner_area = block
            .inner(area)
            .centered_horizontally(Constraint::Length(horizontal_size))
            .centered_vertically(Constraint::Length(vertical_size));

        if self.event.is_some() {
            // Constraitns(bot, mid, top)  = calendar, pad, event
            let vertical = Layout::vertical([Constraint::Length(7), Constraint::Length(1), Constraint::Length(2)]);
            let [top_area, _mid_area, bot_area] = inner_area.layout(&vertical);

            // TODO: Make the top area for the rectangle if there's space

            // TODO: Make the bottom box for the events name if there's space
            self.render_calendar(top_area, buf);
            self.render_event_details(bot_area, buf);
            block.render(area, buf);
            return;
        }

        // TODO: Make the bottom box for the events name if there's space
        self.render_calendar(inner_area, buf);
        block.render(area, buf);
    }
}

impl CalendarView {

    pub fn render_size_warning(&self, area: Rect, buf: &mut Buffer) {
        let warning_text = format!("need min {}x{}", MIN_CALENDAR_WIDTH, MIN_CALENDAR_HEIGHT);
        let block = Block::bordered();
        let inner_area = block.inner(area).centered(
            Constraint::Length(warning_text.len() as u16),
            Constraint::Length(1),
        );

        let text = Text::from(warning_text);
        Paragraph::new(text)
            .wrap(Wrap { trim: true })
            .render(inner_area, buf);
    }

    pub fn render_calendar(&self, area: Rect, buf: &mut Buffer) {
        let col_constraints = (0..7).map(|_| Constraint::Length(4));
        let row_constraits = (0..7).map(|_| Constraint::Length(1));
        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraits).spacing(0);

        let rows = vertical.split(area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());
        let header_cells = cells.clone().take(7).collect::<Vec<_>>();
        let weeks_cells = cells.clone().skip(7).collect::<Vec<_>>();
        self.render_weeks_header(&header_cells, buf);
        self.render_days(&weeks_cells, buf, self.start_day as i32, self.n_days);
    }

    pub fn render_event_details(&self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]);
        let [title_area, desc_area] = area.layout(&vertical);

        let event = self.event.as_ref().unwrap();
        let event_dt = format!("{}", event.start.format("%H:%M:%S"));
        let title = Line::from_iter([
            Span::from(&event.title).blue().bold(),
            Span::from(event_dt).blue(),
        ]);
        title.render(title_area, buf);

        let desc = Line::from(Span::from(&event.desc).green());
        desc.render(desc_area, buf);
    }

    pub fn render_weeks_header(&self, row: &[Rect], buf: &mut Buffer) {
        let header_style = Style::default().bold().fg(Color::Blue);

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
            Paragraph::new(text).render(cell, buf);
        }

        // Styled days afterwards
        for (day, style) in self.styled_days.clone() {
            let cell = cells[(start_day + day as i32 - 1) as usize];
            let text = Text::styled(format!("{}", day), style);
            Paragraph::new(text).render(cell, buf);
        }
    }
}

// TODO: We can add some tests to see if the size warning was drawn
