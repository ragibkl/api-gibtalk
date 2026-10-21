mod index;
mod templates;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use http::Method;
use index::SymbolIndex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use templates::{TemplateOutput, TemplateSummary, Templates};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use url::Url;

#[derive(Clone)]
struct AppState {
    index: Arc<SymbolIndex>,
    templates: Arc<Templates>,
}

fn base_url() -> Url {
    let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:3000/".to_string());
    Url::parse(&base_url).unwrap()
}

#[derive(Serialize, Deserialize, Debug)]
struct SymbolsSearchQuery {
    q: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SymbolOutput {
    url: String,
}

async fn symbols_search(
    State(state): State<AppState>,
    Query(query): Query<SymbolsSearchQuery>,
) -> Json<Vec<SymbolOutput>> {
    let base_url = base_url();

    let results = state.index.search(&query.q, 100);

    let result = results
        .into_iter()
        .filter_map(|symbol| {
            let url = base_url.join(&symbol.file).ok()?.to_string();
            Some(SymbolOutput { url })
        })
        .collect();

    Json(result)
}

async fn templates_list(State(state): State<AppState>) -> Json<Vec<TemplateSummary>> {
    Json(state.templates.list(&base_url()))
}

async fn templates_get(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<TemplateOutput>, StatusCode> {
    state
        .templates
        .get(&id, &base_url())
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn health() -> &'static str {
    "ok"
}

fn build_router() -> Router {
    let state = AppState {
        index: Arc::new(SymbolIndex::load("./media")),
        templates: Arc::new(Templates::load("./templates", "./media")),
    };

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/symbols/search/", get(symbols_search))
        .route("/api/templates", get(templates_list))
        .route("/api/templates/:id", get(templates_get))
        .nest_service("/media/", ServeDir::new("media"))
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = build_router();

    tracing::info!("listening on 0.0.0.0:3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_health() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..], b"ok");
    }

    #[tokio::test]
    async fn test_search_returns_ok() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/symbols/search/?q=ketupat")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let results: Vec<SymbolOutput> = serde_json::from_slice(&body).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].url.contains("ketupat"));
    }

    #[tokio::test]
    async fn test_search_by_tag() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/symbols/search/?q=cookies")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let results: Vec<SymbolOutput> = serde_json::from_slice(&body).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.url.contains("kuih-raya")));
    }

    #[tokio::test]
    async fn test_media_fetch() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/media/custom/play/my-turn.png")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        // PNG magic bytes
        assert_eq!(&body[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[tokio::test]
    async fn test_search_then_fetch() {
        let app = build_router();

        // Search for a symbol
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/symbols/search/?q=ketupat")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let results: Vec<SymbolOutput> = serde_json::from_slice(&body).unwrap();
        assert!(!results.is_empty());

        // Extract the path from the first result URL and fetch it
        let url = Url::parse(&results[0].url).unwrap();
        let path = url.path();

        let response = app
            .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        assert_eq!(&body[..8], b"\x89PNG\r\n\x1a\n");
    }

    #[tokio::test]
    async fn test_search_missing_query_returns_400() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/symbols/search/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    async fn get_json<T: serde::de::DeserializeOwned>(
        app: Router,
        uri: &str,
    ) -> (StatusCode, Option<T>) {
        let response = app
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        (status, serde_json::from_slice(&body).ok())
    }

    #[tokio::test]
    async fn test_templates_list() {
        let (status, list) =
            get_json::<Vec<TemplateSummary>>(build_router(), "/api/templates").await;
        assert_eq!(status, StatusCode::OK);
        let list = list.unwrap();
        assert_eq!(list.len(), 17);
        let greetings = list.iter().find(|t| t.id == "greetings-ms").unwrap();
        assert_eq!(greetings.name, "Ucapan");
        assert_eq!(greetings.language, "ms");
        assert!(greetings.words > 0);
        assert!(greetings.url.ends_with("/api/templates/greetings-ms"));
    }

    #[tokio::test]
    async fn test_template_get_and_fetch_pictures() {
        let app = build_router();
        let (status, template) =
            get_json::<TemplateOutput>(app.clone(), "/api/templates/greetings-ms").await;
        assert_eq!(status, StatusCode::OK);
        let template = template.unwrap();
        assert_eq!(template.id, "greetings-ms");
        assert!(!template.words.is_empty());

        // Every picture is a full URL that the API serves, spaces encoded.
        fn uris(words: &[templates::WordOutput], out: &mut Vec<String>) {
            for w in words {
                out.push(w.uri.clone());
                if let Some(c) = &w.children {
                    uris(c, out);
                }
            }
        }
        let mut all = Vec::new();
        uris(&template.words, &mut all);
        assert!(all.iter().any(|u| u.contains("%20")));
        for uri in all {
            assert!(uri.starts_with("http://localhost:3000/media/"), "{}", uri);
            let path = Url::parse(&uri).unwrap().path().to_string();
            let response = app
                .clone()
                .oneshot(Request::builder().uri(&path).body(Body::empty()).unwrap())
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK, "{}", path);
        }
    }

    #[tokio::test]
    async fn test_template_unknown() {
        let (status, _) = get_json::<TemplateOutput>(build_router(), "/api/templates/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    #[should_panic(expected = "picture not found")]
    fn test_template_missing_picture_refuses_to_load() {
        let dir = std::env::temp_dir().join(format!("api-gibtalk-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("index.yaml"),
            "- id: broken\n  name: Broken\n  description: x\n  author: x\n  language: en\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("broken.yaml"),
            "- label: x\n  uri: nope/missing.png\n",
        )
        .unwrap();
        Templates::load(dir.to_str().unwrap(), "./media");
    }
}
