use askama::Template;
use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use tower_http::services::{ServeDir, ServeFile};

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
            page_metadata: PageMetadata,
        }
        let page_metadata = PageMetadata {
            title: "Error - mic0.dev",
            description: "This page has encountered an error",
            uri: "",
            thumbnail: "social_thumb.png",
        };

        let status = match &self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Render(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        let tmpl = Tmpl {
            err: self,
            page_metadata,
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
        .route_service("/robots.txt", ServeFile::new("public/robots.txt"))        
        .route("/sitemap.xml", get(sitemap_handler))
        .route("/", get(index_handler))
        .route("/cv", get(cv_handler))
        .route("/articles", get(articles_handler))
        .route("/articles/{slug}", get(article_handler))
        .fallback(|| async { AppError::NotFound })
}

#[derive(Debug)]
pub struct PageMetadata {
    title: &'static str,
    description: &'static str,
    thumbnail: &'static str,
    uri: &'static str,
}

async fn index_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "index.html")]
    struct Tmpl {
        page_metadata: PageMetadata,
        articles: &'static [ArticleMeta],
    }
    let page_metadata = PageMetadata {
        title: "Home - mic0.dev",
        description: "My homepage and articles on the things im interested in",
        uri: "",
        thumbnail: "social_thumb.png",
    };
    let template = Tmpl {
        articles: get_article_metas(),
        page_metadata,
    };
    Ok(Html(template.render()?))
}

async fn cv_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "cv.html")]
    struct Tmpl {
        page_metadata: PageMetadata,
    }
    let page_metadata = PageMetadata {
        title: "CV - mic0.dev",
        description: "CV of mic0",
        uri: "cv",
        thumbnail: "social_thumb.png",
    };

    let template = Tmpl { page_metadata };
    Ok(Html(template.render()?))
}

async fn articles_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "articles.html")]
    struct Tmpl {
        articles: &'static [ArticleMeta],
        page_metadata: PageMetadata,
    }
    let page_metadata = PageMetadata {
        title: "Articles - mic0.dev",
        description: "CV of mic0",
        uri: "cv",
        thumbnail: "social_thumb.png",
    };

    let template = Tmpl {
        page_metadata,
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
        title: &'static str,
        content: &'static str,
        date_str: &'static str,
        signature_to_display: u8,
        next_article_slug: &'static str,
        next_article_title: &'static str,
        page_metadata: PageMetadata
    }
    let page_metadata = PageMetadata {
      title: &article.title,
      description: article
            .seo_meta
            .description
            .unwrap_or("written by mic0"),
      uri: article.article_uri,
      thumbnail: article.seo_meta.thumbnail.unwrap_or("social_thumb.png")
    };
    let template = Tmpl {
        title: &article.title,
        content: &article.content,
        date_str: &article.date_str,
        signature_to_display: article.signature_to_display,
        next_article_slug: &article.next_article_slug,
        next_article_title: &article.next_article_title,
        page_metadata
    };
    Ok(Html(template.render()?))
}

async fn sitemap_handler() -> Result<impl IntoResponse, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "sitemap.xml")]
    struct Tmpl {
        articles: &'static [ArticleMeta],
    }
    let body = Tmpl { articles: get_article_metas() }.render()?;
    Ok(([(header::CONTENT_TYPE, "application/xml")], body))
}
