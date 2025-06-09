use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use sqlx::{Row, Sqlite, migrate::MigrateDatabase, query, sqlite::SqlitePool};
use std::fs::create_dir_all;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "todo-cli")]
#[command(version = "0.1.0")]
#[command(about = "A CLI and TUI application for managing your todo list")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task to the todo list
    Add { task: String },
    /// List all tasks in the todo list
    List,
    /// Remove a task from the todo list
    Remove { id: u32 },
    /// Mark a task as complete
    Complete { id: u32 },
    /// Reset all tasks
    Reset,
}

#[derive(Debug)]
struct Task {
    id: i64,
    name: String,
    is_done: bool,
}

#[derive(Debug, PartialEq)]
enum InputMode {
    Normal,
    Adding,
    Editing,
}

#[derive(Debug, PartialEq)]
enum AppState {
    TodoList,
    DoneList,
}

struct App {
    pool: SqlitePool,
    tasks: Vec<Task>,
    todo_state: ListState,
    done_state: ListState,
    input: String,
    input_mode: InputMode,
    app_state: AppState,
    show_help: bool,
    editing_task_id: Option<i64>,
}

impl App {
    async fn new() -> Result<Self, sqlx::Error> {
        let pool = Self::initialize_database().await?;
        let tasks = Self::load_tasks(&pool).await?;

        let mut app = App {
            pool,
            tasks,
            todo_state: ListState::default(),
            done_state: ListState::default(),
            input: String::new(),
            input_mode: InputMode::Normal,
            app_state: AppState::TodoList,
            show_help: false,
            editing_task_id: None,
        };

        if !app.get_todo_tasks().is_empty() {
            app.todo_state.select(Some(0));
        }

        Ok(app)
    }

    async fn initialize_database() -> Result<SqlitePool, sqlx::Error> {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_dir = home_dir.join("todo_db");
        create_dir_all(&db_dir).unwrap();

        let db_path = db_dir.join("todo.db");
        let db_url = format!("sqlite://{}", db_path.display());

        if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
            Sqlite::create_database(&db_url).await?;
        }

        let pool = SqlitePool::connect(&db_url).await?;

        query(
            "CREATE TABLE IF NOT EXISTS todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                date_added DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_done INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    async fn load_tasks(pool: &SqlitePool) -> Result<Vec<Task>, sqlx::Error> {
        let rows = query("SELECT id, name, is_done FROM todo ORDER BY id")
            .fetch_all(pool)
            .await?;

        let tasks = rows
            .into_iter()
            .map(|row| Task {
                id: row.get("id"),
                name: row.get("name"),
                is_done: row.get::<i64, _>("is_done") == 1,
            })
            .collect();

        Ok(tasks)
    }

    fn get_todo_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| !task.is_done).collect()
    }

    fn get_done_tasks(&self) -> Vec<&Task> {
        self.tasks.iter().filter(|task| task.is_done).collect()
    }

    async fn add_task(&mut self, task_name: &str) -> Result<(), sqlx::Error> {
        query("INSERT INTO todo (name) VALUES (?)")
            .bind(task_name)
            .execute(&self.pool)
            .await?;

        self.tasks = Self::load_tasks(&self.pool).await?;
        Ok(())
    }

    async fn toggle_task(&mut self, task_id: i64) -> Result<(), sqlx::Error> {
        let task = self.tasks.iter().find(|t| t.id == task_id);
        if let Some(task) = task {
            let new_status = if task.is_done { 0 } else { 1 };
            query("UPDATE todo SET is_done = ? WHERE id = ?")
                .bind(new_status)
                .bind(task_id)
                .execute(&self.pool)
                .await?;

            self.tasks = Self::load_tasks(&self.pool).await?;
        }
        Ok(())
    }

    async fn delete_task(&mut self, task_id: i64) -> Result<(), sqlx::Error> {
        query("DELETE FROM todo WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        self.tasks = Self::load_tasks(&self.pool).await?;
        Ok(())
    }

    async fn update_task(&mut self, task_id: i64, new_name: &str) -> Result<(), sqlx::Error> {
        query("UPDATE todo SET name = ? WHERE id = ?")
            .bind(new_name)
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        self.tasks = Self::load_tasks(&self.pool).await?;
        Ok(())
    }

    fn get_selected_task_id(&self) -> Option<i64> {
        match self.app_state {
            AppState::TodoList => {
                let todo_tasks = self.get_todo_tasks();
                self.todo_state
                    .selected()
                    .and_then(|i| todo_tasks.get(i).map(|task| task.id))
            }
            AppState::DoneList => {
                let done_tasks = self.get_done_tasks();
                self.done_state
                    .selected()
                    .and_then(|i| done_tasks.get(i).map(|task| task.id))
            }
        }
    }

    fn next_task(&mut self) {
        match self.app_state {
            AppState::TodoList => {
                let todo_tasks = self.get_todo_tasks();
                if !todo_tasks.is_empty() {
                    let i = match self.todo_state.selected() {
                        Some(i) => (i + 1) % todo_tasks.len(),
                        None => 0,
                    };
                    self.todo_state.select(Some(i));
                }
            }
            AppState::DoneList => {
                let done_tasks = self.get_done_tasks();
                if !done_tasks.is_empty() {
                    let i = match self.done_state.selected() {
                        Some(i) => (i + 1) % done_tasks.len(),
                        None => 0,
                    };
                    self.done_state.select(Some(i));
                }
            }
        }
    }

    fn previous_task(&mut self) {
        match self.app_state {
            AppState::TodoList => {
                let todo_tasks = self.get_todo_tasks();
                if !todo_tasks.is_empty() {
                    let i = match self.todo_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                todo_tasks.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.todo_state.select(Some(i));
                }
            }
            AppState::DoneList => {
                let done_tasks = self.get_done_tasks();
                if !done_tasks.is_empty() {
                    let i = match self.done_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                done_tasks.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.done_state.select(Some(i));
                }
            }
        }
    }
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Input/Status
        ])
        .margin(1)
        .split(f.area());

    // Title with modern styling
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "Todo TUI",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" - "),
        Span::styled("Press '?' for help", Style::default().fg(Color::Yellow)),
        Span::raw(" | "),
        Span::styled("'q' to quit", Style::default().fg(Color::Red)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title_alignment(ratatui::layout::Alignment::Center),
    );
    f.render_widget(title, chunks[0]);

    // Main content - split into two columns with margin
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .margin(1)
        .split(chunks[1]);

    // Todo list with modern styling
    let todo_tasks = app.get_todo_tasks();
    let todo_items: Vec<ListItem> = todo_tasks
        .iter()
        .map(|task| {
            ListItem::new(Line::from(vec![
                Span::styled("○ ", Style::default().fg(Color::Blue)),
                Span::raw(task.name.clone()),
            ]))
        })
        .collect();

    let todo_list = List::new(todo_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled(
                        "Todo",
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({})", todo_tasks.len())),
                ]))
                .border_style(if app.app_state == AppState::TodoList {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Blue)
                }),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(todo_list, main_chunks[0], &mut app.todo_state.clone());

    // Done list with modern styling
    let done_tasks = app.get_done_tasks();
    let done_items: Vec<ListItem> = done_tasks
        .iter()
        .map(|task| {
            ListItem::new(Line::from(vec![
                Span::styled("✓ ", Style::default().fg(Color::Green)),
                Span::styled(task.name.clone(), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let done_list = List::new(done_items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Line::from(vec![
                    Span::styled(
                        "Done",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(format!(" ({})", done_tasks.len())),
                ]))
                .border_style(if app.app_state == AppState::DoneList {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Green)
                }),
        )
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .fg(Color::Yellow),
        )
        .highlight_symbol("▶ ");

    f.render_stateful_widget(done_list, main_chunks[1], &mut app.done_state.clone());

    // Status bar with modern styling
    let status_text = match app.input_mode {
        InputMode::Normal => {
            let mode_style = Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD);
            let key_style = Style::default().fg(Color::Yellow);
            let text_style = Style::default().fg(Color::White);

            Line::from(vec![
                Span::styled("NORMAL", mode_style),
                Span::raw(" | "),
                Span::styled("j/k", key_style),
                Span::styled(": navigate", text_style),
                Span::raw(" | "),
                Span::styled("h/l", key_style),
                Span::styled(": switch lists", text_style),
                Span::raw(" | "),
                Span::styled("x", key_style),
                Span::styled(": toggle", text_style),
                Span::raw(" | "),
                Span::styled("n", key_style),
                Span::styled(": new task", text_style),
            ])
        }
        InputMode::Adding => Line::from(vec![
            Span::styled(
                "ADDING",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": save | "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel | "),
            Span::styled("Task: ", Style::default().fg(Color::White)),
            Span::styled(app.input.clone(), Style::default().fg(Color::Cyan)),
        ]),
        InputMode::Editing => Line::from(vec![
            Span::styled(
                "EDITING",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            Span::styled("Enter", Style::default().fg(Color::Yellow)),
            Span::raw(": save | "),
            Span::styled("Esc", Style::default().fg(Color::Yellow)),
            Span::raw(": cancel | "),
            Span::styled("Task: ", Style::default().fg(Color::White)),
            Span::styled(app.input.clone(), Style::default().fg(Color::Cyan)),
        ]),
    };

    let status = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title("Status"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(status, chunks[2]);

    // Help popup with modern styling
    if app.show_help {
        let popup_area = centered_rect(60, 70, f.area());
        f.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from(vec![Span::styled(
                "Help - Todo TUI",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Navigation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  k      - Move up"),
            Line::from("  j      - Move down"),
            Line::from("  h      - Switch to Todo list"),
            Line::from("  l      - Switch to Done list"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from("  n      - Add new task"),
            Line::from("  x      - Mark task as done/undone"),
            Line::from("  e      - Edit selected task"),
            Line::from("  d      - Delete selected task"),
            Line::from("  ?      - Show/hide this help"),
            Line::from("  q      - Quit"),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Press any key to close help",
                Style::default().fg(Color::DarkGray),
            )]),
        ];

        let help_popup = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Help")
                    .title_alignment(ratatui::layout::Alignment::Center),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(help_popup, popup_area);
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

async fn run_tui() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new().await?;
    let res = run_app(&mut terminal, &mut app).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

// Fixed: Removed generic parameter and type annotations
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('?') => app.show_help = !app.show_help,
                        KeyCode::Char('n') => {
                            app.input_mode = InputMode::Adding;
                            app.input.clear();
                        }
                        KeyCode::Char('j') | KeyCode::Down => app.next_task(),
                        KeyCode::Char('k') | KeyCode::Up => app.previous_task(),
                        KeyCode::Char('h') | KeyCode::Left => {
                            app.app_state = AppState::TodoList;
                            if !app.get_todo_tasks().is_empty()
                                && app.todo_state.selected().is_none()
                            {
                                app.todo_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            app.app_state = AppState::DoneList;
                            if !app.get_done_tasks().is_empty()
                                && app.done_state.selected().is_none()
                            {
                                app.done_state.select(Some(0));
                            }
                        }
                        KeyCode::Char('x') => {
                            if let Some(task_id) = app.get_selected_task_id() {
                                let _ = app.toggle_task(task_id).await;
                            }
                        }
                        KeyCode::Char('d') => {
                            if let Some(task_id) = app.get_selected_task_id() {
                                let _ = app.delete_task(task_id).await;
                            }
                        }
                        KeyCode::Char('e') => {
                            if let Some(task_id) = app.get_selected_task_id() {
                                app.editing_task_id = Some(task_id);
                                app.input_mode = InputMode::Editing;
                                if let Some(task) = app.tasks.iter().find(|t| t.id == task_id) {
                                    app.input = task.name.clone();
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::Adding => match key.code {
                        KeyCode::Enter => {
                            if !app.input.trim().is_empty() {
                                let task_name = app.input.clone();
                                let _ = app.add_task(&task_name).await;
                            }
                            app.input.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input.clear();
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Enter => {
                            if !app.input.trim().is_empty() {
                                if let Some(task_id) = app.editing_task_id {
                                    let task_name = app.input.clone();
                                    let _ = app.update_task(task_id, &task_name).await;
                                }
                            }
                            app.input.clear();
                            app.editing_task_id = None;
                            app.input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            app.input.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input.pop();
                        }
                        KeyCode::Esc => {
                            app.input.clear();
                            app.editing_task_id = None;
                            app.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    },
                }

                if app.show_help && app.input_mode == InputMode::Normal {
                    app.show_help = false;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Some(_command) => {
            // Fixed: Prefixed with underscore to indicate intentional non-use
            println!("CLI mode: Use without arguments to start TUI mode");
            println!("Example: cargo run");
        }
        None => {
            run_tui().await?;
        }
    }

    Ok(())
}
