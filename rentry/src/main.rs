use clap::{Parser, Subcommand};
use reqwest::{
    Client,
    header::{ACCEPT_ENCODING, HeaderMap, HeaderValue, REFERER, USER_AGENT},
};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use url::Url;

const BASE_PROTOCOL: &str = "https://";
const BASE_URL: &str = "rentry.co";
const SUCCESS_STATUS: &str = "200";

#[derive(Error, Debug)]
enum RentryError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("API error: {0}")]
    Api(String, Vec<String>),
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Clone)]
struct Entry {
    url: String,
    edit_code: String,
    text: String,
}

#[derive(Clone)]
struct UrllibClient {
    client: Client,
    csrf_token: Option<String>,
    csrf_token_time: Option<SystemTime>,
}

impl UrllibClient {
    fn new(timeout: u64) -> Result<Self, RentryError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate"));
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
            ),
        );
        headers.insert(
            reqwest::header::ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.5"),
        );
        headers.insert(
            reqwest::header::UPGRADE_INSECURE_REQUESTS,
            HeaderValue::from_static("1"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout))
            .default_headers(headers)
            .build()
            .map_err(|e| RentryError::Request(e))?;

        Ok(UrllibClient {
            client,
            csrf_token: None,
            csrf_token_time: None,
        })
    }

    async fn get(
        &self,
        url: &str,
        headers: Option<HeaderMap>,
    ) -> Result<reqwest::Response, RentryError> {
        let mut request = self.client.get(url);
        if let Some(h) = headers {
            request = request.headers(h);
        }
        Ok(request.send().await?)
    }

    async fn post(
        &self,
        url: &str,
        data: HashMap<&str, String>,
        headers: Option<HeaderMap>,
    ) -> Result<reqwest::Response, RentryError> {
        let mut request = self.client.post(url).form(&data);
        if let Some(h) = headers {
            request = request.headers(h);
        }
        Ok(request.send().await?)
    }
}

struct RentryClient {
    client: UrllibClient,
    csrf_token_ttl: u64,
    max_retries: u32,
}

impl RentryClient {
    fn new(max_retries: u32) -> Result<Self, RentryError> {
        Ok(RentryClient {
            client: UrllibClient::new(30)?,
            max_retries,
            csrf_token_ttl: 3600,
        })
    }

    async fn get_csrf_token(&mut self) -> Result<String, RentryError> {
        let current_time = SystemTime::now();
        if let (Some(token), Some(time)) = (&self.client.csrf_token, self.client.csrf_token_time) {
            if current_time.duration_since(time).unwrap().as_secs() < self.csrf_token_ttl {
                return Ok(token.clone());
            }
        }

        let url = format!("{}{}", BASE_PROTOCOL, BASE_URL);
        let response = self.client.get(&url, None).await?;

        // Print response status and headers for debugging
        eprintln!("CSRF token request status: {}", response.status());
        eprintln!("CSRF token request headers: {:#?}", response.headers());

        let cookies = response.headers().get_all("set-cookie");
        let token = cookies
            .iter()
            .find_map(|c| {
                c.to_str().ok().and_then(|s| {
                    s.split(';')
                        .find(|p| p.contains("csrftoken="))
                        .map(|p| p.replace("csrftoken=", ""))
                })
            })
            .ok_or_else(|| RentryError::Api("Failed to get CSRF token".into(), vec![]))?;

        self.client.csrf_token = Some(token.clone());
        self.client.csrf_token_time = Some(current_time);
        Ok(token)
    }

    async fn retry<F, Fut, T>(&self, f: F) -> Result<T, RentryError>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, RentryError>>,
    {
        let mut last_error = None;
        for attempt in 0..self.max_retries {
            match f().await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.max_retries - 1 {
                        tokio::time::sleep(Duration::from_secs(2u64.pow(attempt))).await;
                    }
                }
            }
        }
        Err(last_error.unwrap())
    }

    async fn get_raw(&mut self, url: &str) -> Result<String, RentryError> {
        if url.is_empty() {
            return Err(RentryError::Validation("URL is required".into()));
        }

        let endpoint = format!("{}{}/api/raw/{}", BASE_PROTOCOL, BASE_URL, url);
        let client = self.client.clone();

        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(reqwest::header::ORIGIN, HeaderValue::from_static(BASE_URL));

        let response = self
            .retry(|| async {
                let resp = client.get(&endpoint, Some(headers.clone())).await?;

                // Print response status and headers for debugging
                eprintln!("Get raw request status: {}", resp.status());
                eprintln!("Get raw request headers: {:#?}", resp.headers());

                // Get response body as bytes
                let bytes = resp.bytes().await?;
                let text = String::from_utf8_lossy(&bytes);
                eprintln!("Get raw response body: {}", text);

                // Try to parse the JSON
                serde_json::from_slice::<Value>(&bytes).map_err(|e| {
                    RentryError::Api(format!("Failed to parse JSON response: {}", e), vec![])
                })
            })
            .await?;

        if response["status"] != SUCCESS_STATUS {
            return Err(RentryError::Api(
                format!("Failed to get raw content: {}", response["content"]),
                vec![],
            ));
        }

        Ok(response["content"].as_str().unwrap().to_string())
    }

    async fn create_entry(&mut self, entry: Entry) -> Result<Entry, RentryError> {
        if entry.text.is_empty() {
            return Err(RentryError::Validation("Text is required".into()));
        }

        let csrftoken = self.get_csrf_token().await?;
        let mut payload = HashMap::new();
        payload.insert("csrfmiddlewaretoken", csrftoken);
        payload.insert("url", entry.url.clone());
        payload.insert("edit_code", entry.edit_code.clone());
        payload.insert("text", entry.text.clone());

        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_static(BASE_URL));
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(reqwest::header::ORIGIN, HeaderValue::from_static(BASE_URL));
        headers.insert(
            reqwest::header::HeaderName::from_static("x-requested-with"),
            HeaderValue::from_static("XMLHttpRequest"),
        );

        let client = self.client.clone();
        let url = format!("{}{}/api/new", BASE_PROTOCOL, BASE_URL);

        let response = self
            .retry(|| async {
                let resp = client
                    .post(&url, payload.clone(), Some(headers.clone()))
                    .await?;

                // Print response status and headers for debugging
                eprintln!("Create entry request status: {}", resp.status());
                eprintln!("Create entry request headers: {:#?}", resp.headers());

                // Get response body as bytes
                let bytes = resp.bytes().await?;
                let text = String::from_utf8_lossy(&bytes);
                eprintln!("Create entry response body: {}", text);

                // Try to parse the JSON
                serde_json::from_slice::<Value>(&bytes).map_err(|e| {
                    RentryError::Api(format!("Failed to parse JSON response: {}", e), vec![])
                })
            })
            .await?;

        if response["status"] != SUCCESS_STATUS {
            let errors = response["errors"]
                .as_str()
                .unwrap_or("")
                .split('.')
                .filter(|e| !e.is_empty())
                .map(String::from)
                .collect();
            return Err(RentryError::Api(
                format!("Failed to create entry: {}", response["content"]),
                errors,
            ));
        }

        Ok(Entry {
            url: response["url"].as_str().unwrap().to_string(),
            edit_code: response["edit_code"].as_str().unwrap().to_string(),
            text: entry.text,
        })
    }

    async fn edit_entry(&mut self, entry: Entry) -> Result<(), RentryError> {
        if entry.url.is_empty() {
            return Err(RentryError::Validation("URL is required".into()));
        }
        if entry.edit_code.is_empty() {
            return Err(RentryError::Validation("Edit code is required".into()));
        }
        if entry.text.is_empty() {
            return Err(RentryError::Validation("Text is required".into()));
        }

        let csrftoken = self.get_csrf_token().await?;
        let mut payload = HashMap::new();
        payload.insert("csrfmiddlewaretoken", csrftoken);
        payload.insert("edit_code", entry.edit_code.clone());
        payload.insert("text", entry.text.clone());

        let mut headers = HeaderMap::new();
        headers.insert(REFERER, HeaderValue::from_static(BASE_URL));
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            HeaderValue::from_static("application/json"),
        );
        headers.insert(reqwest::header::ORIGIN, HeaderValue::from_static(BASE_URL));

        let client = self.client.clone();
        let url = format!("{}{}/api/edit/{}", BASE_PROTOCOL, BASE_URL, entry.url);

        let response = self
            .retry(|| async {
                let resp = client
                    .post(&url, payload.clone(), Some(headers.clone()))
                    .await?;

                // Print response status and headers for debugging
                eprintln!("Edit entry request status: {}", resp.status());
                eprintln!("Edit entry request headers: {:#?}", resp.headers());

                // Get response body as bytes
                let bytes = resp.bytes().await?;
                let text = String::from_utf8_lossy(&bytes);
                eprintln!("Edit entry response body: {}", text);

                // Try to parse the JSON
                serde_json::from_slice::<Value>(&bytes).map_err(|e| {
                    RentryError::Api(format!("Failed to parse JSON response: {}", e), vec![])
                })
            })
            .await?;

        if response["status"] != SUCCESS_STATUS {
            let errors = response["errors"]
                .as_str()
                .unwrap_or("")
                .split('.')
                .filter(|e| !e.is_empty())
                .map(String::from)
                .collect();
            return Err(RentryError::Api(
                format!("Failed to edit entry: {}", response["content"]),
                errors,
            ));
        }

        Ok(())
    }
}

#[derive(Subcommand)]
enum Command {
    #[clap(about = "Create a new entry")]
    New {
        #[clap(short, long)]
        url: Option<String>,
        #[clap(short = 'p', long = "edit-code")]
        edit_code: Option<String>,
        text: Option<String>,
    },
    #[clap(about = "Edit an existing entry")]
    Edit {
        #[clap(short, long)]
        url: String,
        #[clap(short = 'p', long = "edit-code")]
        edit_code: String,
        text: Option<String>,
    },
    #[clap(about = "Get the raw content of an entry")]
    Raw {
        #[clap(short, long)]
        url: String,
    },
}

#[derive(Parser)]
#[clap(about = "A command-line client for rentry.co paste service")]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut client = RentryClient::new(3)?;

    match args.command {
        Command::New {
            url,
            edit_code,
            text,
        } => {
            let text = text.unwrap_or_else(|| {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).unwrap();
                input.trim().to_string()
            });

            if text.is_empty() {
                eprintln!("No text provided");
                std::process::exit(1);
            }

            let entry = Entry {
                url: url.unwrap_or_default(),
                edit_code: edit_code.unwrap_or_default(),
                text,
            };

            match client.create_entry(entry).await {
                Ok(result) => println!("Url: {}\nEdit code: {}", result.url, result.edit_code),
                Err(e) => {
                    eprintln!("{}", e);
                    if let RentryError::Api(_, errors) = &e {
                        for error in errors {
                            eprintln!("{}", error);
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::Edit {
            url,
            edit_code,
            text,
        } => {
            let text = text.unwrap_or_else(|| {
                let mut input = String::new();
                io::stdin().read_to_string(&mut input).unwrap();
                input.trim().to_string()
            });

            if text.is_empty() {
                eprintln!("No text provided");
                std::process::exit(1);
            }

            let entry = Entry {
                url: Url::parse(&url).map_or(url.clone(), |u| {
                    u.path().trim_start_matches('/').to_string()
                }),
                edit_code,
                text,
            };

            match client.edit_entry(entry).await {
                Ok(_) => println!("Ok"),
                Err(e) => {
                    eprintln!("{}", e);
                    if let RentryError::Api(_, errors) = &e {
                        for error in errors {
                            eprintln!("{}", error);
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::Raw { url } => {
            let url = Url::parse(&url).map_or(url.clone(), |u| {
                u.path().trim_start_matches('/').to_string()
            });
            match client.get_raw(&url).await {
                Ok(content) => println!("{}", content),
                Err(e) => {
                    eprintln!("{}", e);
                    if let RentryError::Api(_, errors) = &e {
                        for error in errors {
                            eprintln!("{}", error);
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
