use askama::Template;
use axum::{
    Router,
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use tower_http::services::ServeDir;

use crate::article::{ArticleMeta, get_article_metas};

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Page not found")]
    NotFound,
    #[error("Display error: {0}")]
    Render(#[from] askama::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        #[derive(Debug, Template)]
        #[template(path = "error.html")]
        struct Tmpl {
            err: AppError,
        }
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let tmpl = Tmpl { err: self };
        if let Ok(body) = tmpl.render() {
            (status, Html(body)).into_response()
        } else {
            (status, "Something went wrong").into_response()
        }
    }
}

pub(crate) fn create_router() -> Router {
    Router::new()
        .nest_service("/public", ServeDir::new("public"))
        .route("/", get(index_handler))
        .route("/cv", get(cv_handler))
        .route("/articles", get(articles_handler))
        .route("/articles/{slug}", get(article_handler))
        .fallback(|| async { AppError::NotFound })
}

async fn index_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "index.html")]
    struct Tmpl {
        articles: Vec<ArticleMeta>,
    }
    let template = Tmpl {
        articles: get_article_metas(),
    };
    Ok(Html(template.render()?))
}

async fn cv_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "cv.html")]
    struct Tmpl {}
    let template = Tmpl {};
    Ok(Html(template.render()?))
}

async fn articles_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "articles.html")]
    struct Tmpl {
        articles: Vec<ArticleMeta>,
    }
    let template = Tmpl {
        articles: get_article_metas(),
    };
    Ok(Html(template.render()?))
}

async fn article_handler(Path(slug): Path<String>) -> Result<impl IntoResponse, AppError> {
    let all_articles = get_article_metas();
    let article = all_articles
        .iter()
        .find(|a| a.slug.eq(&slug))
        .ok_or(AppError::NotFound)?;

    #[derive(Debug, Template)]
    #[template(path = "_article.html")]
    struct Tmpl {
        title: String,
        content: String,
        date_str: String,
        signature_to_display: u8,
    }
    let template = Tmpl {
        title: article.title.clone(),
        content: article.content.clone(),
        date_str: article.date_str.clone(),
        signature_to_display: article.signature_to_display,
    };
    Ok(Html(template.render()?))
}
