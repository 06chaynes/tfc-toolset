use tfc_toolset::workspace::Workspace;
use tui::{
    layout::{Alignment, Constraint},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, Paragraph, Row, Table,
    },
};

use crate::{App, InputMode};

pub fn render<'a>(
    workspace_list: Vec<Workspace>,
    app: &mut App,
) -> (Paragraph<'a>, List<'a>, Table<'a>, Table<'a>, List<'a>) {
    let filter_style: Style;
    match app.input_mode {
        InputMode::Navigation => {
            filter_style = Style::default();
            let info_text = vec![
                Span::raw("Press "),
                Span::styled(
                    "ctrl + f",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to edit filter."),
            ];
            app.info = Paragraph::new(Text::from(Spans::from(info_text)))
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Info")
                        .border_type(BorderType::Plain),
                );
        }
        InputMode::Editing => {
            filter_style = Style::default().fg(Color::Yellow);
            let info_text = vec![
                Span::raw("Press "),
                Span::styled(
                    "Esc",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to cancel, "),
                Span::styled(
                    "Enter",
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(" to apply the filter"),
            ];
            app.info = Paragraph::new(Text::from(Spans::from(info_text)))
                .style(Style::default().fg(Color::Yellow))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Info")
                        .border_type(BorderType::Plain),
                );
        }
    }
    let filtered_list = workspace_list
        .into_iter()
        .filter(|workspace| {
            workspace.attributes.name.contains(&app.applied_workspace_filter)
        })
        .collect::<Vec<_>>();
    app.workspace_count = filtered_list.len();
    let filter = Paragraph::new(app.workspace_filter.clone())
        .style(filter_style)
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    let workspaces = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Workspaces")
        .border_type(BorderType::Plain);
    let items: Vec<_> = filtered_list
        .iter()
        .map(|workspace| {
            ListItem::new(Spans::from(vec![Span::styled(
                workspace.attributes.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_workspace: Option<Workspace> = filtered_list
        .get(app.workspace_list_state.selected().unwrap_or(0))
        .cloned();

    let list = List::new(items).block(workspaces).highlight_style(
        Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let mut workspace_tags: Vec<ListItem> = vec![];

    if let Some(workspace) = selected_workspace {
        for tag in workspace.attributes.tag_names {
            workspace_tags.push(ListItem::new(tag));
        }
        let tag_list = List::new(workspace_tags).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Tags")
                .border_type(BorderType::Plain),
        );

        let vcs_table = match workspace.attributes.vcs_repo {
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
            Cell::from(Span::raw(workspace.id.to_string())),
            Cell::from(Span::raw(workspace.attributes.name)),
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
    } else {
        let tag_list = List::new(vec![]).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Tags")
                .border_type(BorderType::Plain),
        );

        let blank_row = Row::new(vec![Cell::from(Span::raw("".to_string()))]);

        let vcs_table =
            Table::new(vec![Row::new(vec![Cell::from(Span::raw(""))])])
                .header(blank_row.clone())
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("VCS")
                        .border_type(BorderType::Plain),
                )
                .widths(&[
                    Constraint::Percentage(80),
                    Constraint::Percentage(20),
                ]);

        let workspace_detail = Table::new(vec![Row::new(vec![Cell::from(
            Span::raw("No Workspace Selected".to_string()),
        )])])
        .header(blank_row)
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
}
