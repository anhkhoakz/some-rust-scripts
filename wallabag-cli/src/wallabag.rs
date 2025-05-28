use crate::config::Config;
use crate::utils;
use colored::Colorize;
use reqwest::{Client, Error, Response};
use serde_json::value;

pub async fn login() {
    let mut config: Config = match Config::load() {
        Some(cfg) => {
            println!("Config loaded: {:?}", cfg);
            cfg
        }

        None => {
            println!("Config not found. Please set up your config file.");
            return;
        }
    };
    // TODO: Check if config is valid, parse both http, https, and none
    let client: Client = Client::new();
    let url: String = format!("{}/oauth/v2/token", config.base_url.trim_end_matches('/'));
    let params: [(&str, &str); 5] = [
        ("grant_type", "password"),
        ("client_id", config.client_id.as_str()),
        ("client_secret", config.client_secret.as_str()),
        ("username", config.username.as_str()),
        ("password", config.password.as_str()),
    ];

    let resp: Result<Response, Error> = client.post(url).form(&params).send().await;

    match resp {
        Ok(r) => {
            if r.status().is_success() {
                match serde_json::from_str::<serde_json::Value>(&r.text().await.unwrap_or_default())
                {
                    Ok(json) => {
                        config.access_token = json
                            .get("access_token")
                            .and_then(|v: &value::Value| v.as_str())
                            .map(|s: &str| s.to_string());
                        config.refresh_token = json
                            .get("refresh_token")
                            .and_then(|v: &value::Value| v.as_str())
                            .map(|s: &str| s.to_string());
                        config.save();
                        println!("Login successful. Tokens saved.");
                    }
                    Err(e) => eprintln!("Failed to parse response: {}", e),
                }
            } else {
                println!("Login failed: {}", r.status());
                if let Ok(text) = r.text().await {
                    println!("Response: {}", text);
                }
            }
        }
        Err(e) => eprintln!("Request error: {}", e),
    }
}

pub async fn add_entry(url: &str) {
    // TODO: Use config, send POST to /api/entries
    println!("Add entry not implemented: {}", url);
}

pub async fn get_entries(
    archive: Option<u8>,
    starred: Option<u8>,
    sort: Option<&str>,
    order: Option<&str>,
    page: Option<u32>,
    per_page: Option<u32>,
    tags: Option<&str>,
    since: Option<u64>,
    public: Option<u8>,
    detail: Option<&str>,
    domain_name: Option<&str>,
) {
    let archive: u8 = archive.unwrap_or(0);
    let starred: u8 = starred.unwrap_or(0);
    let sort: &str = sort.unwrap_or("created");
    let order: &str = order.unwrap_or("desc");
    let page: u32 = page.unwrap_or(1);
    let per_page: u32 = per_page.unwrap_or(30);
    let tags: &str = tags.unwrap_or("");
    let since: u64 = since.unwrap_or(0);
    let public: u8 = public.unwrap_or(0);
    let detail: &str = detail.unwrap_or("full");
    let domain_name: &str = domain_name.unwrap_or("");

    if !utils::validate_archive(archive) {
        println!("Invalid value for 'archive'. Only 0 or 1 allowed.");
        return;
    }
    if !utils::validate_starred(starred) {
        println!("Invalid value for 'starred'. Only 0 or 1 allowed.");
        return;
    }
    if !utils::validate_public(public) {
        println!("Invalid value for 'public'. Only 0 or 1 allowed.");
        return;
    }
    if !utils::validate_sort(sort) {
        println!("Invalid value for 'sort'. Only 'created', 'updated', or 'archived' allowed.");
        return;
    }
    if !utils::validate_order(order) {
        println!("Invalid value for 'order'. Only 'asc' or 'desc' allowed.");
        return;
    }
    if !utils::validate_detail(detail) {
        println!("Invalid value for 'detail'. Only 'metadata' or 'full' allowed.");
        return;
    }

    let config: Config = match Config::load() {
        Some(cfg) => cfg,
        None => {
            println!("Config not found. Please set up your config file.");
            return;
        }
    };
    let access_token: &String = match &config.access_token {
        Some(token) => token,
        None => {
            println!("No access token found. Please login first.");
            return;
        }
    };

    let client: Client = Client::new();
    let url: String = format!("{}/api/entries", config.base_url.trim_end_matches('/'));
    let query_params = utils::build_query_params(
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
    );

    let resp: Result<Response, Error> = client
        .get(&url)
        .bearer_auth(access_token)
        .query(&query_params)
        .send()
        .await;

    if resp.is_err() {
        eprintln!("Request error: {}", resp.unwrap_err());
        return;
    }

    let json = match utils::handle_response(resp.unwrap()).await {
        Ok(j) => j,
        Err(e) => {
            if e == "invalid_grant" {
                login().await;
            }
            eprintln!("{}", e);
            return;
        }
    };

    let items: Option<&Vec<value::Value>> = json
        .get("_embedded")
        .and_then(|e: &value::Value| e.get("items"))
        .and_then(|i: &value::Value| i.as_array());

    if items.is_none() {
        println!("No entries found.");
        return;
    }

    for item in items.unwrap() {
        let title: Option<&str> = item.get("title").and_then(|t: &value::Value| t.as_str());
        let id = item.get("id").and_then(|i: &value::Value| i.as_u64());
        if title.is_none() || id.is_none() {
            continue;
        }
        let id: u32 = id.unwrap() as u32;
        let url: &str = item
            .get("url")
            .and_then(|u: &value::Value| u.as_str())
            .unwrap_or("N/A");
        let archive: u8 = item
            .get("archive")
            .and_then(|a: &value::Value| a.as_u64())
            .unwrap_or(0) as u8;
        let starred: u8 = item
            .get("starred")
            .and_then(|s: &value::Value| s.as_u64())
            .unwrap_or(0) as u8;

        utils::print_entry(id, title.unwrap(), url, archive, starred);
    }
}

pub async fn search_entries(query: &str) {
    // TODO: Use config, send GET to /api/entries?search=...
    println!("Search not implemented: {}", query);
}

pub async fn get_entry(id: u32) {
    // TODO: Use config, send GET to /api/entries/{id}
    println!("Read entry not implemented: {}", id);
}

pub async fn delete_entry(id: u32) {
    // TODO: Use config, send DELETE to /api/entries/{id}
    println!("Delete entry not implemented: {}", id);
}
