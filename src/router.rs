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
            page_title: String,
            page_description: String,
            uri: String,
        }
        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let tmpl = Tmpl {
            page_title: "mic0.dev - Error".to_string(),
            page_description: status.to_string(),
            err: self,
            uri: "error".to_string(),
        };
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
        page_title: String,
        page_description: String,
        uri: String,
        articles: &'static [ArticleMeta],
    }
    let template = Tmpl {
        page_title: "mic0.dev".to_string(),
        page_description: "Hey, im mic0! Welcome to my home page.".to_string(),
        articles: get_article_metas(),
        uri: "".to_string(),
    };
    Ok(Html(template.render()?))
}

async fn cv_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "cv.html")]
    struct Tmpl {
        page_title: String,
        page_description: String,
        uri: String,
    }
    let template = Tmpl {
        page_title: "mic0.dev - CV".to_string(),
        page_description: "CV of mic0".to_string(),
        uri: "cv".to_string(),
    };
    Ok(Html(template.render()?))
}

async fn articles_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "articles.html")]
    struct Tmpl {
        page_title: String,
        page_description: String,
        articles: &'static [ArticleMeta],
        uri: String,
    }
    let template = Tmpl {
        page_title: "mic0.dev - Articles".to_string(),
        page_description: "Articles by mic0".to_string(),
        articles: get_article_metas(),
        uri: "articles".to_string(),
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
        page_title: &'static str,
        page_description: String,
        title: &'static str,
        content: &'static str,
        date_str: &'static str,
        signature_to_display: u8,
        next_article_slug: &'static str,
        next_article_title: &'static str,
        uri: String,
    }
    let template = Tmpl {
        page_title: &article.title,
        page_description: format!("written by mic0 on {}", article.date_str),
        title: &article.title,
        content: &article.content,
        date_str: &article.date_str,
        signature_to_display: article.signature_to_display,
        next_article_slug: &article.next_article_slug,
        next_article_title: &article.next_article_title,
        uri: format!("articles/{}", article.slug.clone()),
    };
    Ok(Html(template.render()?))
}
