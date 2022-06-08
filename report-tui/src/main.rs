mod home;
mod info;
mod report;
mod workspaces;

use report::TfcReport;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode,
        KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use miette::IntoDiagnostic;
use std::io;
use thiserror::Error;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, ListState, Paragraph, Tabs},
    Frame, Terminal,
};

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

pub enum InputMode {
    Navigation,
    Editing,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Info,
    Workspaces,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Info => 1,
            MenuItem::Workspaces => 2,
        }
    }
}

pub struct App {
    /// Current input mode
    input_mode: InputMode,
    /// Current page
    active_nav_item: MenuItem,
    /// Loaded report
    report: TfcReport,
    /// Current value of the workspace filter input box
    workspace_filter: String,
    /// Applied value of the workspace filter input box
    applied_workspace_filter: String,
    /// Count of workspaces in report
    workspace_count: usize,
    /// State data for workspaces list
    workspace_list_state: ListState,
}

fn main() -> miette::Result<()> {
    enable_raw_mode().into_diagnostic()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .into_diagnostic()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).into_diagnostic()?;

    // create app and run it
    let mut workspace_list_state = ListState::default();
    workspace_list_state.select(Some(0));
    let report = report::read()?;
    let workspace_count = count_workspaces(&report);
    let app = App {
        input_mode: InputMode::Navigation,
        report,
        workspace_filter: String::new(),
        applied_workspace_filter: String::new(),
        workspace_count,
        workspace_list_state,
        active_nav_item: MenuItem::Home,
    };
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode().into_diagnostic()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)
        .into_diagnostic()?;
    terminal.show_cursor().into_diagnostic()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Navigation => match key {
                    KeyEvent {
                        code: KeyCode::Char('Q'),
                        modifiers: KeyModifiers::SHIFT,
                    } => {
                        return Ok(());
                    }
                    KeyEvent {
                        code: KeyCode::Char('f'),
                        modifiers: KeyModifiers::CONTROL,
                    } => app.input_mode = InputMode::Editing,
                    KeyEvent {
                        code: KeyCode::Char('H'),
                        modifiers: KeyModifiers::SHIFT,
                    } => app.active_nav_item = MenuItem::Home,
                    KeyEvent {
                        code: KeyCode::Char('I'),
                        modifiers: KeyModifiers::SHIFT,
                    } => app.active_nav_item = MenuItem::Info,
                    KeyEvent {
                        code: KeyCode::Char('W'),
                        modifiers: KeyModifiers::SHIFT,
                    } => app.active_nav_item = MenuItem::Workspaces,
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    } => match app.active_nav_item {
                        MenuItem::Home => {}
                        MenuItem::Info => {}
                        MenuItem::Workspaces => {
                            if let Some(selected) =
                                app.workspace_list_state.selected()
                            {
                                if selected >= app.workspace_count - 1 {
                                    app.workspace_list_state.select(Some(0));
                                } else {
                                    app.workspace_list_state
                                        .select(Some(selected + 1));
                                }
                            }
                        }
                    },
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    } => match app.active_nav_item {
                        MenuItem::Home => {}
                        MenuItem::Info => {}
                        MenuItem::Workspaces => {
                            if let Some(selected) =
                                app.workspace_list_state.selected()
                            {
                                if selected > 0 {
                                    app.workspace_list_state
                                        .select(Some(selected - 1));
                                } else {
                                    app.workspace_list_state
                                        .select(Some(app.workspace_count - 1));
                                }
                            }
                        }
                    },
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.applied_workspace_filter =
                            app.workspace_filter.clone();
                        app.input_mode = InputMode::Navigation;
                    }
                    KeyCode::Esc => {
                        app.workspace_filter =
                            app.applied_workspace_filter.clone();
                        app.input_mode = InputMode::Navigation;
                    }
                    KeyCode::Char(c) => {
                        app.workspace_filter.push(c);
                    }
                    KeyCode::Backspace => {
                        app.workspace_filter.pop();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let menu_titles = vec!["Home", "Info", "Workspaces", "Quit"];
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [Constraint::Length(3), Constraint::Min(2), Constraint::Length(3)]
                .as_ref(),
        )
        .split(size);

    let about = Paragraph::new("report-tui 2022")
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
        .select(app.active_nav_item.into())
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green))
        .divider(Span::raw("|"));

    f.render_widget(tabs, chunks[0]);
    match app.active_nav_item {
        MenuItem::Home => f.render_widget(home::render(), chunks[1]),
        MenuItem::Info => {
            match app.report.clone() {
                TfcReport::Clean(r) => f.render_widget(
                    info::render(
                        serde_json::to_string(&r.reporter).unwrap(),
                        r.report_version,
                        r.bin_version,
                        serde_json::to_string_pretty(&r.meta.query).unwrap(),
                        serde_json::to_string_pretty(&r.meta.pagination)
                            .unwrap(),
                    ),
                    chunks[1],
                ),
                TfcReport::Which(r) => f.render_widget(
                    info::render(
                        serde_json::to_string(&r.reporter).unwrap(),
                        r.report_version,
                        r.bin_version,
                        serde_json::to_string_pretty(&r.meta.query).unwrap(),
                        serde_json::to_string_pretty(&r.meta.pagination)
                            .unwrap(),
                    ),
                    chunks[1],
                ),
            };
        }
        MenuItem::Workspaces => {
            let workspaces_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [Constraint::Percentage(30), Constraint::Percentage(70)]
                        .as_ref(),
                )
                .split(chunks[1]);

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [Constraint::Percentage(10), Constraint::Percentage(90)]
                        .as_ref(),
                )
                .split(workspaces_chunks[0]);

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
            let workspace_list = match app.report.clone() {
                TfcReport::Clean(r) => r.data.workspaces,
                TfcReport::Which(r) => r.data.workspaces,
            };
            let (
                left_filter,
                left_ws_list,
                right_details,
                right_vcs,
                right_tags,
            ) = workspaces::render(workspace_list, app);
            f.render_widget(left_filter, left_chunks[0]);
            f.render_stateful_widget(
                left_ws_list,
                left_chunks[1],
                &mut app.workspace_list_state,
            );
            f.render_widget(right_details, right_chunks[0]);
            f.render_widget(right_vcs, right_chunks[1]);
            f.render_widget(right_tags, right_chunks[2]);
        }
    }
    f.render_widget(about, chunks[2]);
}

pub fn count_workspaces(report: &TfcReport) -> usize {
    match report {
        TfcReport::Clean(r) => r.data.workspaces.len(),
        TfcReport::Which(r) => r.data.workspaces.len(),
    }
}
