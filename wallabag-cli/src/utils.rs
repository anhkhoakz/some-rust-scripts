use colored::Colorize;
use reqwest::{Client, Error, Response};
use serde_json::value;

pub fn validate_archive(archive: u8) -> bool {
    archive == 0 || archive == 1
}

pub fn validate_starred(starred: u8) -> bool {
    starred == 0 || starred == 1
}

pub fn validate_public(public: u8) -> bool {
    public == 0 || public == 1
}

pub fn validate_sort(sort: &str) -> bool {
    ["created", "updated", "archived"].contains(&sort)
}

pub fn validate_order(order: &str) -> bool {
    ["asc", "desc"].contains(&order)
}

pub fn validate_detail(detail: &str) -> bool {
    ["metadata", "full"].contains(&detail)
}

pub fn build_query_params(
    archive: u8,
    starred: u8,
    sort: &str,
    order: &str,
    page: u32,
    per_page: u32,
    tags: &str,
    since: u64,
    public: u8,
    detail: &str,
    domain_name: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("archive", archive.to_string()),
        ("starred", starred.to_string()),
        ("sort", sort.to_string()),
        ("order", order.to_string()),
        ("page", page.to_string()),
        ("perPage", per_page.to_string()),
        ("tags", tags.to_string()),
        ("since", since.to_string()),
        ("public", public.to_string()),
        ("detail", detail.to_string()),
        ("domain_name", domain_name.to_string()),
    ]
}

pub fn print_entry(id: u32, title: &str, url: &str, archive: u8, starred: u8) {
    println!(
        "{} | {} | {} | {} | {}",
        id.to_string().blue(),
        title.green(),
        url.yellow(),
        archive,
        starred
    );
}

pub async fn handle_response(r: Response) -> Result<value::Value, String> {
    if !r.status().is_success() {
        let text = r.text().await.unwrap_or_default();
        if text.contains("\"error\":\"invalid_grant\"") {
            return Err("invalid_grant".to_string());
        }
        return Err(format!(
            "Failed to get entries: {} Response: {}",
            r.status(),
            text
        ));
    }

    let text = r
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    serde_json::from_str::<value::Value>(&text)
        .map_err(|e| format!("Failed to parse response: {}", e))
}
