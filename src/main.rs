use axum::{extract::Query, routing::get, Json, Router};
use http::Method;
use rust_search::{similarity_sort, SearchBuilder};
use serde::{Deserialize, Serialize};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
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
        .map(|s| {
            let url = base_url.join(&s).unwrap().to_string();
            SymbolOutput { url }
        })
        .collect();

    Json(result)
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .layer(cors)
        .route("/api/symbols/search/", get(symbols_search))
        .nest_service("/media/", ServeDir::new("media"));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
