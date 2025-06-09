use clap::Parser;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;

const DEFAULT_PAGE_SIZE: &str = "25";
const CRATES_API_URL: &str = "https://crates.io/api/v1/crates";
const NPM_API_URL: &str = "https://api.npms.io/v2/search";
const DOCKER_API_URL: &str = "https://index.docker.io/v1/search";
const COMPOSER_API_URL: &str = "https://packagist.org/search.json";
const JSDELIVR_API_URL: &str = "https://ofcncog2cu-dsn.algolia.net/1/indexes/npm-search/query";
const SUPPORTED_SOURCES: &[&str] = &["npm", "docker", "jsdelivr", "crates", "composer"];

/// Command-line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Source to search (npm, docker, jsdelivr, crates, composer)
    #[arg()]
    source: String,

    /// Query string to search for
    #[arg()]
    query: String,
}

/// Search crates.io
async fn search_crates(query: &str) -> Result<Value, Box<dyn Error>> {
    let client: Client = Client::new();
    let res: reqwest::Response = client
        .get(CRATES_API_URL)
        .query(&[("q", query), ("page", "1"), ("per_page", DEFAULT_PAGE_SIZE)])
        .header("User-Agent", "my_crawler (help@my_crawler.com)")
        .send()
        .await?;
    Ok(res.json::<Value>().await?)
}

/// Search npm
async fn search_npm(query: &str) -> Result<Value, Box<dyn Error>> {
    let client: Client = Client::new();
    let res: reqwest::Response = client
        .get(NPM_API_URL)
        .query(&[("q", query), ("size", DEFAULT_PAGE_SIZE)])
        .send()
        .await?;
    Ok(res.json::<Value>().await?)
}

/// Search jsDelivr
async fn search_jsdelivr(query: &str) -> Result<Value, Box<dyn Error>> {
    let client: Client = Client::new();
    let attrs: [&str; 4] = ["name", "version", "description", "homepage"];
    let payload: Value = serde_json::json!({
        "params": format!(
            "query={}&page=0&hitsPerPage={}&attributesToHighlight=[]&attributesToRetrieve={}",
            query,
            DEFAULT_PAGE_SIZE,
            serde_json::to_string(&attrs)?
        )
    });
    let res: reqwest::Response = client
        .post(JSDELIVR_API_URL)
        .header(
            "x-algolia-agent",
            "Algolia for JavaScript (3.35.1); Browser (lite)",
        )
        .header("x-algolia-application-id", "OFCNCOG2CU")
        .header("x-algolia-api-key", "f54e21fa3a2a0160595bb058179bfb1e")
        .json(&payload)
        .send()
        .await?;
    let v: Value = res.json::<Value>().await?;
    Ok(v.get("hits")
        .cloned()
        .unwrap_or_else(|| serde_json::json!([])))
}

/// Search Docker Hub
async fn search_docker(query: &str) -> Result<Value, Box<dyn Error>> {
    let client: Client = Client::new();
    let res: reqwest::Response = client
        .get(DOCKER_API_URL)
        .query(&[("q", query), ("page", "1")])
        .send()
        .await?;
    Ok(res.json::<Value>().await?)
}

/// Search Composer (Packagist)
async fn search_composer(query: &str) -> Result<Value, Box<dyn Error>> {
    let client: Client = Client::new();
    let res: reqwest::Response = client
        .get(COMPOSER_API_URL)
        .query(&[("q", query), ("per_page", DEFAULT_PAGE_SIZE)])
        .send()
        .await?;
    Ok(res.json::<Value>().await?)
}

/// Validate the source argument
fn validate_source(source: &str) -> bool {
    SUPPORTED_SOURCES.contains(&source)
}

/// Print error to stderr
fn print_error(msg: &str) {
    eprintln!("{}", msg);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Args = Args::parse();

    if !validate_source(&args.source) {
        print_error(
            "Unsupported source. Supported sources: npm, docker, jsdelivr, crates, composer.",
        );
        std::process::exit(1);
    }

    let result: Result<Value, Box<dyn Error>> = match args.source.as_str() {
        "npm" => search_npm(&args.source).await,
        "docker" => search_docker(&args.query).await,
        "jsdelivr" => search_jsdelivr(&args.query).await,
        "crates" => search_crates(&args.query).await,
        "composer" => search_composer(&args.query).await,
        _ => unreachable!(),
    };

    match result {
        Ok(data) => {
            println!("{}", serde_json::to_string(&data)?);
        }
        Err(error) => {
            print_error(&format!("Error: {}", error));
        }
    }
    Ok(())
}
