use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use tfc_toolset::{
    settings::{Pagination, Query},
    workspace::Workspace,
};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph,
        Row, Table, Tabs,
    },
    Terminal,
};

const DB_PATH: &str = "./report.json";

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Workspaces,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Workspaces => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) =
                    event::read().expect("can read events")
                {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate && tx.send(Event::Tick).is_ok()
            {
                last_tick = Instant::now();
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let menu_titles = vec!["Home", "Workspaces", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut workspace_list_state = ListState::default();
    workspace_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let info = Paragraph::new("report-tui 2022")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title("Info")
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Green))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Workspaces => {
                    let workspaces_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [
                                Constraint::Percentage(30),
                                Constraint::Percentage(70),
                            ]
                            .as_ref(),
                        )
                        .split(chunks[1]);

                    let right_chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Percentage(20),
                                Constraint::Percentage(20),
                                Constraint::Percentage(50),
                            ]
                            .as_ref(),
                        )
                        .split(workspaces_chunks[1]);
                    let (left, right_details, right_vcs, right_tags) =
                        render_workspaces(&workspace_list_state);
                    rect.render_stateful_widget(
                        left,
                        workspaces_chunks[0],
                        &mut workspace_list_state,
                    );
                    rect.render_widget(right_details, right_chunks[0]);
                    rect.render_widget(right_vcs, right_chunks[1]);
                    rect.render_widget(right_tags, right_chunks[2]);
                }
            }
            rect.render_widget(info, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('w') => active_menu_item = MenuItem::Workspaces,
                KeyCode::Down => {
                    if let Some(selected) = workspace_list_state.selected() {
                        let workspaces_count = read_db()
                            .expect("can't fetch workspace list")
                            .len();
                        if selected >= workspaces_count - 1 {
                            workspace_list_state.select(Some(0));
                        } else {
                            workspace_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = workspace_list_state.selected() {
                        let workspaces_count = read_db()
                            .expect("can't fetch workspace list")
                            .len();
                        if selected > 0 {
                            workspace_list_state.select(Some(selected - 1));
                        } else {
                            workspace_list_state
                                .select(Some(workspaces_count - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "workspace-CLI",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'w' to access workspaces.")]),
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

fn render_workspaces<'a>(
    workspace_list_state: &ListState,
) -> (List<'a>, Table<'a>, Table<'a>, List<'a>) {
    let workspaces = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Workspaces")
        .border_type(BorderType::Plain);

    let workspace_list = read_db().expect("can't fetch workspace list");
    let items: Vec<_> = workspace_list
        .iter()
        .map(|workspace| {
            ListItem::new(Spans::from(vec![Span::styled(
                workspace.attributes.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_workspace = workspace_list
        .get(
            workspace_list_state
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

    (list, workspace_detail, vcs_table, tag_list)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Meta {
    pub query: Option<Query>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Data {
    pub workspaces: Vec<Workspace>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Report {
    pub report_version: String,
    pub bin_version: String,
    pub reporter: String,
    pub meta: Meta,
    pub data: Data,
}

fn read_db() -> Result<Vec<Workspace>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Report = serde_json::from_str(&db_content)?;
    Ok(parsed.data.workspaces)
}
