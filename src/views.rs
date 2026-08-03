use ratatui::{buffer::Buffer, layout::{Constraint, Layout, Rect}, style::{Color, Style}, text::{Line, Span, Text}, widgets::{Block, Paragraph, Widget}};

#[derive(Debug)]
pub struct CalendarView {
    pub month_name: String,
    pub start_day: u8,
    pub year: i32,
    pub n_days: usize,

    // TODO: we probably want to make this
    // TODO: a vector or make this struct
    // TODO: contain abunch of styles per
    // TODO: day.
    pub current_day: Option<u8>
}

impl Widget for &CalendarView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_text = format!("{} {}", self.year, self.month_name);
        let block = Block::bordered()
            .title_alignment(ratatui::layout::HorizontalAlignment::Center)
            .title(Line::from(title_text));

        let col_constraints = (0..7).map(|_| Constraint::Length(4));
        let row_constraits = (0..7).map(|_| Constraint::Length(1));

        let horizontal = Layout::horizontal(col_constraints).spacing(0);
        let vertical = Layout::vertical(row_constraits).spacing(0);

        let inner_area = block
            .inner(area)
            .centered_horizontally(Constraint::Length(7 * 4))
            .centered_vertically(Constraint::Length(7));

        let rows = vertical.split(inner_area);
        let cells = rows.iter().flat_map(|&row| horizontal.split(row).to_vec());

        let header_cells = cells.clone().take(7).collect::<Vec<_>>();
        let weeks_cells = cells.clone().skip(7).collect::<Vec<_>>();

        self.render_weeks_header(&header_cells, buf);
        self.render_days(&weeks_cells, buf, self.start_day as i32, self.n_days, self.current_day.map(|d| d as i32));

        block.render(area, buf);
    }
}

impl CalendarView {
    pub fn render_weeks_header(&self, row: &[Rect], buf: &mut Buffer) {
        let header_style = Style::default()
            .bold()
            .fg(Color::Blue);

        for (day, &cell) in ["S", "M", "T", "W", "T", "F", "S"].into_iter().zip(row) {
            Span::styled(day, header_style).render(cell, buf);
        }
    }

    pub fn render_days(&self, cells: &[Rect], buf: &mut Buffer, start_day: i32, n_days: usize, today: Option<i32>) {
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

        if let Some(today) = today {
            let cell = cells[(start_day + today - 1) as usize];
            let text = Text::styled(format!("{}", today), Style::default().bold().blue());
            Paragraph::new(text)
                .render(cell, buf);
        }
    }
}
