use axum::{extract::Query, routing::get, Json, Router};
use http::Method;
use rust_search::{similarity_sort, SearchBuilder};
use serde::{Deserialize, Serialize};
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

#[axum_macros::debug_handler]
async fn symbols_search(Query(query): Query<SymbolsSearchQuery>) -> Json<Vec<SymbolOutput>> {
    let mut search: Vec<String> = SearchBuilder::default()
        .location("./media")
        .search_input(&query.q)
        .limit(100) // results to return
        .ext(".png")
        .depth(3)
        .ignore_case()
        .hidden()
        .build()
        .collect();
    similarity_sort(&mut search, &query.q);

    let base_url = std::env::var("BASE_URL").unwrap_or("http://localhost:3000/".to_string());
    let base_url = Url::parse(&base_url).unwrap();

    let result = search
        .into_iter()
        .filter_map(|s| {
            let url = base_url.join(&s).ok()?.to_string();
            Some(SymbolOutput { url })
        })
        .collect();

    Json(result)
}

async fn health() -> &'static str {
    "ok"
}

fn build_router() -> Router {
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/symbols/search/", get(symbols_search))
        .nest_service("/media/", ServeDir::new("media"))
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
                    .uri("/api/symbols/search/?q=apple")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
        let results: Vec<SymbolOutput> = serde_json::from_slice(&body).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].url.contains("apple"));
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
