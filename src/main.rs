use std::{error::Error, sync::Arc, time::Duration};

use axum::{
    Json, Router,
    extract::{Query, State},
    routing::get,
    serve
};
use reqwest::{
    Client,
    header::{self, AUTHORIZATION, HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str, json};
use tokio::{net::TcpListener, time::sleep};

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyResponse {
    pub data: Vec<Article>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub url: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Debug, Deserialize)]
pub struct Candidate {
    pub content: Content,
}

#[derive(Debug, Deserialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Debug, Deserialize)]
pub struct Part {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CuratedArticle {
    pub title: String,
    pub url: String,
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct TempResponse {
    data: Vec<TempArticle>,
    pagination: TempPagination,
}

#[derive(Debug, Deserialize)]
pub struct TempArticle {
    id: String,
    title: String,
    url: String,
    summary: Option<String>,
    tags: Vec<String>,
    #[serde(rename = "publishedAt")]
    published_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TempPagination {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
    #[serde(rename = "endCursor", default)]
    end_cursor: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SignalParams {
    pub theme: String,
    pub time: String,
}

struct AppState {
    client: Client,
    gemini_key: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 1. Token
    // GANTI BAGIAN TOKEN DI DALAM fn main() MENJADI SEPERTI INI:
    let token = std::env::var("DAILY_DEV_TOKEN")
        .expect("DAILY_DEV_TOKEN should in environment variable");
    let gemini = std::env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY should in environment variable");

    // 2. Prepare header for auth
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", token))?,
    );

    // 3. create http client
    let client = Client::builder().default_headers(headers).build()?;
    let shared_state = Arc::new(AppState {
        client,
        gemini_key: gemini.to_string(),
    });

    let app = Router::new()
        .route("/api/signal", get(handle_curation))
        .with_state(shared_state);

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    println!("Backend 'The Signal' Active in http://localhost:8000");
    serve(listener, app).await?;

    Ok(())
}

async fn handle_curation(
    Query(params): Query<SignalParams>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CuratedArticle>>, String> {
    let mut all_articles: Vec<Article> = Vec::new();
    let mut cursor: Option<String> = None;

    let time_filter = match params.time.to_lowercase().as_str() {
        "day" | "week" | "month" | "year" | "all" => params.time.to_lowercase(),
        _ => "week".to_string(),
    };

    println!("Loading Get Data From Daily.dev...");

    loop {
        // 4. Call Endpoint post new
        let base_url = format!(
            "https://api.daily.dev/public/v1/search/posts?q={}&sort=DATE&time={}&limit=50",
            params.theme, time_filter
        );
        let url = match &cursor {
            Some(c) => format!("{}&cursor={}", base_url, c),
            None => base_url,
        };

        let respon = state
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let page_data: Value = respon.json().await.map_err(|e| e.to_string())?;

        if let Some(data_array) = page_data["data"].as_array() {
            if data_array.is_empty() {
                break;
            }

            for item in data_array {
                all_articles.push(Article {
                    id: item["id"].as_str().unwrap_or("").to_string(),
                    title: item["title"].as_str().unwrap_or("").to_string(),
                    url: item["url"].as_str().unwrap_or("").to_string(),
                    summary: item["summary"].as_str().map(|s| s.to_string()),
                    tags: item["tags"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|v| v.as_str().unwrap_or("").to_string())
                        .collect(),
                });
            }
        }

        println!(
            "----- Successdfully secured {} Article...",
            all_articles.len()
        );

        let has_next_page = page_data["pagination"]["hasNextPage"]
            .as_bool()
            .unwrap_or(false);
        let end_cursor = page_data["pagination"]["cursor"]
            .as_str()
            .map(|s| s.to_string());

        if !has_next_page || end_cursor.is_none() {
            break;
        }
        cursor = end_cursor;
        sleep(Duration::from_millis(500)).await;
    }

    if all_articles.is_empty() {
        return Err("Article is missing.".to_string());
    }

    let result = filter_article(&all_articles, &state.gemini_key)
        .await
        .map_err(|e| e.to_string())?;
    let gemini_json: GeminiResponse = from_str(&result).map_err(|e| e.to_string())?;
    // UBAH BLOK AKHIR FUNCTION MENJADI SEPERTI INI:
    if let Some(candidate) = gemini_json.candidates.first() {
        if let Some(part) = candidate.content.parts.first() {
            let clean_text = part
                .text
                .trim()
                .trim_start_matches("```json")
                .trim_end_matches("```")
                .trim();
            let article_data: Vec<CuratedArticle> =
                from_str(clean_text).map_err(|e| e.to_string())?;

            return Ok(Json(article_data));
        }
    }

    Err("Failed Getting Recommendation Content From AI".to_string())
}

async fn filter_article(articles: &[Article], gemini_key: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let url =
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash:generateContent";

    let mut headers = HeaderMap::new();
    headers.insert("x-goog-api-key", HeaderValue::from_str(gemini_key)?);
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application.json"),
    );

    let prompt_text = format!(
        "You are a cold, highly efficient senior tech news curator. \
         Your job is to filter this list of articles and select a MAXIMUM of 3 posts that are the most fundamental, \
         high-impact on performance, or feature clean code architecture (focus heavily on Rust/Backend). \
         Ignore any articles regarding community drama, tech birthdays/anniversaries, or beginner-friendly tutorials.\n\n\
         Article Data:\n{}\n\n\
         Return the response as a JSON ARRAY ONLY, without markdown wrapping, and no text outside the JSON. \
         The object format MUST look exactly like this: \
         [{{\"title\": \"...\", \"url\": \"...\", \"reason\": \"a 1-sentence reason why this is important for senior devs\"}}]",
        serde_json::to_string(articles)?
    );

    let request_body = json!({
        "contents": [{
            "parts": [{
                "text": prompt_text
            }]
        }]
    });

    println!("Loading Send data to gemini for filtering,,");
    let response = client
        .post(url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;

    let response_text = response.text().await?;
    Ok(response_text)
}
