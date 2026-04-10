mod index;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use http::Method;
use index::SymbolIndex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
    trace::TraceLayer,
};
use url::Url;

#[derive(Serialize, Deserialize, Debug)]
struct SymbolsSearchQuery {
    q: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct SymbolOutput {
    url: String,
}

async fn symbols_search(
    State(index): State<Arc<SymbolIndex>>,
    Query(query): Query<SymbolsSearchQuery>,
) -> Json<Vec<SymbolOutput>> {
    let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:3000/".to_string());
    let base_url = Url::parse(&base_url).unwrap();

    let results = index.search(&query.q, 100);

    let result = results
        .into_iter()
        .filter_map(|symbol| {
            let url = base_url.join(&symbol.file).ok()?.to_string();
            Some(SymbolOutput { url })
        })
        .collect();

    Json(result)
}

async fn health() -> &'static str {
    "ok"
}

fn build_router() -> Router {
    let index = Arc::new(SymbolIndex::load("./media"));

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/symbols/search/", get(symbols_search))
        .nest_service("/media/", ServeDir::new("media"))
        .with_state(index)
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
        assert!(results[0].url.contains("kuih-raya"));
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
}
