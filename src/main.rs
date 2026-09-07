mod article;
mod router;

use std::sync::LazyLock;

use router::create_router;

use crate::article::ARTICLES;

#[tokio::main]
async fn main() {
    LazyLock::force(&ARTICLES); // force it here so it dies on startup if wrong
    let app = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
