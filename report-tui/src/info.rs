use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render<'a>(
    reporter: String,
    report_version: String,
    bin_version: String,
    query: String,
    pagination: String,
) -> Paragraph<'a> {
    Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Reporter: "),
            Span::styled(reporter, Style::default().fg(Color::Green)),
        ]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Reporter Version: "),
            Span::styled(bin_version, Style::default().fg(Color::LightCyan)),
        ]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![
            Span::raw("Report Version: "),
            Span::styled(report_version, Style::default().fg(Color::LightCyan)),
        ]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Query: ")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            query,
            Style::default().fg(Color::LightCyan),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Pagination: ")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            pagination,
            Style::default().fg(Color::LightCyan),
        )]),
        Spans::from(vec![Span::raw("")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Info")
            .border_type(BorderType::Plain),
    )
}
