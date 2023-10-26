use ratatui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Welcome")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("to")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::styled(
            "report-tui",
            Style::default().fg(Color::Green),
        )]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'shift + i' to access report info.")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'shift + w' to access workspaces.")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'shift + h' to access this page.")]),
        Line::from(vec![Span::raw("")]),
        Line::from(vec![Span::raw("Press 'shift + q' to quit.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
    );
    home
}
