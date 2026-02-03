use axum::{http::StatusCode, response::Html, Router};
use tower_http::services::ServeDir;
use askama_axum::{IntoResponse, Template};

use crate::templates::*;

pub async fn index() -> impl IntoResponse
{
    let template = IndexTemplate {
        title: "Louis | Software Engineer",
    };
    let reply_html = template.render().expect("Failed to render template");
    (StatusCode::OK, Html(reply_html).into_response())
}


pub fn serve_static_files() -> Router
{
    Router::new().nest_service("/", ServeDir::new("public"))
}

pub async fn error_handler() -> impl IntoResponse
{
    let template = ErrorTemplate {
        status: 404,
        message: "Not found".to_owned(),
    };
    let reply_html = template.render().expect("Failed to render template");
    (StatusCode::NOT_FOUND, Html(reply_html).into_response())
}
