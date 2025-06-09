use clap::{Parser, Subcommand};
use sqlx::{Row, Sqlite, migrate::MigrateDatabase, query, sqlite::SqlitePool};
use std::fs::create_dir_all;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "todo-cli")]
#[command(version = "0.1.0")]
#[command(about = "A CLI application for managing your todo list with SQLite")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new task to the todo list
    Add {
        /// The task to add
        task: String,
    },
    /// List all tasks in the todo list
    List,
    /// Remove a task from the todo list
    Remove {
        /// The task ID to remove
        id: u32,
    },
    /// Mark a task as complete
    Complete {
        /// The task ID to mark as complete
        id: u32,
    },
    /// Reset all tasks
    Reset,
}

// Custom error type for better error handling
#[derive(Debug)]
enum TodoError {
    Database(sqlx::Error),
    TaskNotFound(u32),
    InvalidInput(String),
}

impl From<sqlx::Error> for TodoError {
    fn from(err: sqlx::Error) -> Self {
        TodoError::Database(err)
    }
}

impl std::fmt::Display for TodoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoError::Database(err) => write!(f, "Database error: {}", err),
            TodoError::TaskNotFound(id) => write!(f, "Task with ID {} not found", id),
            TodoError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for TodoError {}

struct TodoApp {
    pool: SqlitePool,
}

impl TodoApp {
    async fn new() -> Result<Self, TodoError> {
        let db_url = Self::get_database_url()?;
        let pool = Self::create_connection(&db_url).await?;
        Self::initialize_schema(&pool).await?;

        Ok(TodoApp { pool })
    }

    fn get_database_url() -> Result<String, TodoError> {
        let home_dir: PathBuf = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let db_dir: PathBuf = home_dir.join("todo_db");

        create_dir_all(&db_dir)
            .map_err(|_| TodoError::InvalidInput("Cannot create database directory".to_string()))?;

        let db_path = db_dir.join("todo.db");
        Ok(format!("sqlite://{}", db_path.display()))
    }

    async fn create_connection(db_url: &str) -> Result<SqlitePool, TodoError> {
        if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
            Sqlite::create_database(db_url).await?;
        }

        let pool = SqlitePool::connect(db_url).await?;
        Ok(pool)
    }

    async fn initialize_schema(pool: &SqlitePool) -> Result<(), TodoError> {
        query(
            "CREATE TABLE IF NOT EXISTS todo (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                date_added DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_done INTEGER NOT NULL DEFAULT 0
            )",
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn add_task(&self, task_name: &str) -> Result<(), TodoError> {
        if task_name.trim().is_empty() {
            return Err(TodoError::InvalidInput(
                "Task name cannot be empty".to_string(),
            ));
        }

        query("INSERT INTO todo (name) VALUES (?)")
            .bind(task_name.trim())
            .execute(&self.pool)
            .await?;

        println!("✓ Added task: {}", task_name);
        Ok(())
    }

    async fn list_tasks(&self) -> Result<(), TodoError> {
        let rows = query("SELECT id, name, is_done FROM todo ORDER BY id")
            .fetch_all(&self.pool)
            .await?;

        if rows.is_empty() {
            println!("No tasks found. Add some tasks to get started!");
            return Ok(());
        }

        println!("Todo List:");
        println!("-----------");

        for row in rows {
            let id: i64 = row.get("id");
            let name: String = row.get("name");
            let is_done: i64 = row.get("is_done");

            let status_icon = if is_done == 1 { "✓" } else { "○" };
            println!("{} [{}] {}", status_icon, id, name);
        }

        Ok(())
    }

    async fn remove_task(&self, task_id: u32) -> Result<(), TodoError> {
        let result = query("DELETE FROM todo WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TodoError::TaskNotFound(task_id));
        }

        println!("✓ Removed task with ID: {}", task_id);
        Ok(())
    }

    async fn complete_task(&self, task_id: u32) -> Result<(), TodoError> {
        let result = query("UPDATE todo SET is_done = 1 WHERE id = ?")
            .bind(task_id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(TodoError::TaskNotFound(task_id));
        }

        println!("✓ Marked task {} as complete", task_id);
        Ok(())
    }

    async fn reset_all_tasks(&self) -> Result<(), TodoError> {
        query("DELETE FROM todo").execute(&self.pool).await?;

        println!("✓ All tasks have been deleted");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let app = match TodoApp::new().await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Failed to initialize todo app: {}", e);
            std::process::exit(1);
        }
    };

    let result = match args.command {
        Commands::Add { task } => app.add_task(&task).await,
        Commands::List => app.list_tasks().await,
        Commands::Remove { id } => app.remove_task(id).await,
        Commands::Complete { id } => app.complete_task(id).await,
        Commands::Reset => app.reset_all_tasks().await,
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_empty_task_fails() {
        let app: TodoApp = TodoApp::new().await.unwrap();
        let result: Result<(), TodoError> = app.add_task("").await;
        assert!(matches!(result, Err(TodoError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_task_fails() {
        let app: TodoApp = TodoApp::new().await.unwrap();
        let result: Result<(), TodoError> = app.remove_task(999).await;
        assert!(matches!(result, Err(TodoError::TaskNotFound(999))));
    }
}
