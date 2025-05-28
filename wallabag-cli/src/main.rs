mod config;
mod utils;
mod wallabag;

use clap::{Parser, Subcommand};
use tokio::runtime;

#[derive(Parser)]
#[command(name = "wallabag-cli")]
#[command(about = "CLI for Wallabag", long_about = None)]
#[command(author = "anhkhoakz")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}
#[derive(Subcommand)]
enum Commands {
    /// Log in to your Wallabag account
    Login,
    /// Add a new entry by URL
    Add {
        /// The URL to add
        url: String,
    },
    /// List all saved entries
    List {
        /// Filter by archive status (0 or 1)
        #[arg(long)]
        archive: Option<u8>,
        /// Filter by starred status (0 or 1)
        #[arg(long)]
        starred: Option<u8>,
        /// Sort by field (created, updated, archived)
        #[arg(long)]
        sort: Option<String>,
        /// Order (asc or desc)
        #[arg(long)]
        order: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Entries per page
        #[arg(long)]
        per_page: Option<u32>,
        /// Filter by tags (comma separated)
        #[arg(long)]
        tags: Option<String>,
        /// Filter by since (timestamp)
        #[arg(long)]
        since: Option<u64>,
        /// Filter by public status (0 or 1)
        #[arg(long)]
        public: Option<u8>,
        /// Detail level (metadata or full)
        #[arg(long)]
        detail: Option<String>,
        /// Filter by domain name
        #[arg(long)]
        domain_name: Option<String>,
    },
    /// Search entries by query
    Search {
        /// The search query
        query: String,
    },
    /// Read an entry by its ID
    Read {
        /// The entry ID
        id: u32,
    },
    /// Delete an entry by its ID
    Delete {
        /// The entry ID
        id: u32,
    },
}

fn main() {
    let rt: runtime::Runtime = runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let cli: Cli = Cli::parse();

        match cli.command {
            Commands::Login => {
                wallabag::login().await;
            }
            // Commands::Logout => {
            //     wallabag::logout().await;
            // }
            Commands::Add { url } => {
                wallabag::add_entry(&url).await;
            }
            Commands::List {
                archive,
                starred,
                sort,
                order,
                page,
                per_page,
                tags,
                since,
                public,
                detail,
                domain_name,
            } => {
                wallabag::get_entries(
                    archive,
                    starred,
                    sort.as_deref(),
                    order.as_deref(),
                    page,
                    per_page,
                    tags.as_deref(),
                    since,
                    public,
                    detail.as_deref(),
                    domain_name.as_deref(),
                )
                .await;
            }
            Commands::Search { query } => {
                wallabag::search_entries(&query).await;
            }
            Commands::Read { id } => {
                wallabag::get_entry(id).await;
            }
            Commands::Delete { id } => {
                wallabag::delete_entry(id).await;
            }
        }
    });
}
