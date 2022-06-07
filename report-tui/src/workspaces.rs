use tfc_toolset::workspace::Workspace;
use tui::{
    layout::Constraint,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table,
    },
};

use crate::{App, InputMode};

pub fn render<'a>(
    workspace_list: Vec<Workspace>,
    app: &App,
) -> (Paragraph<'a>, List<'a>, Table<'a>, Table<'a>, List<'a>) {
    let filter = Paragraph::new(app.workspace_filter.clone())
        .style(match app.input_mode {
            InputMode::Navigation => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    let workspaces = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Workspaces")
        .border_type(BorderType::Plain);
    let items: Vec<_> = workspace_list
        .iter()
        .filter(|workspace| {
            workspace.attributes.name.contains(&app.workspace_filter)
        })
        .map(|workspace| {
            ListItem::new(Spans::from(vec![Span::styled(
                workspace.attributes.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_workspace = workspace_list
        .get(
            app.workspace_list_state
                .selected()
                .expect("there is always a selected workspace"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(workspaces).highlight_style(
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut workspace_tags: Vec<ListItem> = vec![];
    for tag in selected_workspace.attributes.tag_names {
        workspace_tags.push(ListItem::new(tag));
    }
    let tag_list = List::new(workspace_tags).block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Tags")
            .border_type(BorderType::Plain),
    );

    let vcs_table = match selected_workspace.attributes.vcs_repo {
        Some(v) => Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(v.repository_http_url)),
            Cell::from(Span::raw(v.branch)),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "URL",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Branch",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("VCS")
                .border_type(BorderType::Plain),
        )
        .widths(&[Constraint::Percentage(70), Constraint::Percentage(20)]),
        None => Table::new(vec![Row::new(vec![
            Cell::from(Span::raw("No VCS Attached")),
            Cell::from(Span::raw("")),
        ])])
        .header(Row::new(vec![
            Cell::from(Span::styled(
                "URL",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Cell::from(Span::styled(
                "Branch",
                Style::default().add_modifier(Modifier::BOLD),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("VCS")
                .border_type(BorderType::Plain),
        )
        .widths(&[Constraint::Percentage(80), Constraint::Percentage(20)]),
    };

    let workspace_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_workspace.id.to_string())),
        Cell::from(Span::raw(selected_workspace.attributes.name)),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Details")
            .border_type(BorderType::Plain),
    )
    .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)]);

    (filter, list, workspace_detail, vcs_table, tag_list)
}
