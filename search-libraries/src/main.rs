use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;

pub struct ApiClient {
    search_url: String,
    params: HashMap<String, String>,
    user_agent: Option<String>,
}

impl ApiClient {
    pub fn new(search_url: &str, user_agent: Option<&str>) -> Self {
        Self {
            search_url: search_url.to_string(),
            params: HashMap::new(),
            user_agent: user_agent.map(|ua: &str| ua.to_string()),
        }
    }

    pub fn set_param(mut self, key: &str, value: &str) -> Self {
        self.params.insert(key.to_string(), value.to_string());
        self
    }

    pub async fn get(&self, endpoint: &str) -> Result<Value, Box<dyn Error>> {
        let url: String = format!("{}{}", self.search_url, endpoint);
        let client: Client = Client::new();
        let mut request: reqwest::RequestBuilder = client.get(&url).query(&self.params);

        if let Some(user_agent) = &self.user_agent {
            request = request.header("User-Agent", user_agent);
        }

        let response: reqwest::Response = request.send().await?;
        Ok(response.json().await?)
    }
}

pub async fn search_crates(query: Option<&str>) -> Result<Value, Box<dyn Error>> {
    ApiClient::new(
        "https://crates.io/api/v1/",
        Some("my_crawler (help@my_crawler.com)"),
    )
    .set_param("page", "1")
    .set_param("per_page", "25")
    .set_param("q", query.unwrap_or(""))
    .get("crates")
    .await
}

pub async fn search_npm(query: Option<&str>) -> Result<Value, Box<dyn Error>> {
    ApiClient::new("https://api.npms.io/v2/search/", None)
        .set_param("q", query.unwrap_or(""))
        .set_param("size", "25")
        .get("search")
        .await
}

pub async fn search_jsdelivr(query: Option<&str>) -> Result<Value, Box<dyn Error>> {
    let query: &str = query.unwrap_or("");
    let attributes_to_retrieve: [&str; 4] = ["name", "version", "description", "homepage"];

    let payload: Value = serde_json::json!({
        "params": format!(
            "query={}&page=0&hitsPerPage=25&attributesToHighlight=[]&attributesToRetrieve={}",
            query,
            serde_json::to_string(&attributes_to_retrieve)?
        )
    });

    let response: reqwest::Response = Client::new()
        .post("https://ofcncog2cu-dsn.algolia.net/1/indexes/npm-search/query")
        .header(
            "x-algolia-agent",
            "Algolia for JavaScript (3.35.1); Browser (lite)",
        )
        .header("x-algolia-application-id", "OFCNCOG2CU")
        .header("x-algolia-api-key", "f54e21fa3a2a0160595bb058179bfb1e")
        .json(&payload)
        .send()
        .await?;

    if response.status().is_success() {
        let hits: Value = response
            .json::<Value>()
            .await?
            .get("hits")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        Ok(hits)
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            response.text().await?,
        )))
    }
}

pub async fn search_docker(query: Option<&str>) -> Result<Value, Box<dyn Error>> {
    ApiClient::new("https://index.docker.io/v1/search", None)
        .set_param("q", query.unwrap_or(""))
        .set_param("page", "1")
        .get("")
        .await
}

pub async fn search_composer(query: Option<&str>) -> Result<Value, Box<dyn Error>> {
    ApiClient::new("https://packagist.org/search.json", None)
        .set_param("q", query.unwrap_or(""))
        .set_param("per_page", "25")
        .get("")
        .await
}

pub fn write_json_to_file<T: Serialize>(data: &T, file_name: &str) -> Result<(), Box<dyn Error>> {
    let json_string: String = serde_json::to_string_pretty(data)?;
    File::create(file_name)?.write_all(json_string.as_bytes())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <source> <query>", args[0]);
        return Ok(());
    }

    let source: &String = &args[1];
    let query: &String = &args[2];

    let result: Result<Value, Box<dyn Error>> = match source.as_str() {
        "npm" => search_npm(Some(query)).await,
        "docker" => search_docker(Some(query)).await,
        "jsdelivr" => search_jsdelivr(Some(query)).await,
        "crates" => search_crates(Some(query)).await,
        "composer" => search_composer(Some(query)).await,
        _ => {
            eprintln!("Unsupported source: {}. Supported sources are 'npm', 'docker', 'jsdelivr', 'crates', and 'composer'.", source);
            return Ok(());
        }
    };

    match result {
        Ok(data) => {
            println!("{}", serde_json::to_string_pretty(&data)?);
        }
        Err(error) => {
            let error_response: Value = serde_json::json!({
                "items": [
                    {
                        "title": "Error",
                        "subtitle": error.to_string()
                    }
                ]
            });
            println!("{}", serde_json::to_string_pretty(&error_response)?);
        }
    }
    Ok(())
}
