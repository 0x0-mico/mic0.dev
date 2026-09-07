mod article;
mod router;

use router::create_router;

use crate::article::get_article_metas;

#[tokio::main]
async fn main() {
    get_article_metas(); // call it once so it doesn't die at runtime but at startup
    let app = create_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
