use askama::Template;
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{Html, IntoResponse},
    Router,
};
use serde::Deserialize;
use tower_http::services::ServeDir;

use crate::blog;
use crate::templates::*;

pub async fn index() -> impl IntoResponse
{
    let template = IndexTemplate {
        title: "Louis | Software Engineer",
    };
    let reply_html = template.render().expect("Failed to render template");
    (StatusCode::OK, Html(reply_html).into_response())
}

#[derive(Deserialize)]
pub struct BlogQuery
{
    pub tag: Option<String>,
}

pub async fn blog_index(Query(query): Query<BlogQuery>) -> impl IntoResponse
{
    let mut posts = blog::get_all_posts();
    let tags = blog::get_all_tags();

    if let Some(ref tag) = query.tag {
        posts.retain(|p| p.tags.iter().any(|t| t == tag));
    }

    let template = BlogTemplate {
        title: "Louis | Blog",
        posts,
        tags,
        active_tag: query.tag,
    };
    let reply_html = template.render().expect("Failed to render template");
    (StatusCode::OK, Html(reply_html).into_response())
}

pub async fn blog_post(Path(slug): Path<String>) -> impl IntoResponse
{
    match blog::get_post_by_slug(&slug) {
        Some(post) => {
            let template = PostTemplate {
                title: "Louis | Blog",
                post,
            };
            let reply_html = template.render().expect("Failed to render template");
            (StatusCode::OK, Html(reply_html).into_response())
        }
        None => {
            let template = ErrorTemplate {
                status: 404,
                message: "Post not found".to_owned(),
            };
            let reply_html = template.render().expect("Failed to render template");
            (StatusCode::NOT_FOUND, Html(reply_html).into_response())
        }
    }
}

pub async fn latest_posts() -> impl IntoResponse
{
    let posts: Vec<_> = blog::get_all_posts().into_iter().take(3).collect();
    let template = LatestPostsTemplate { posts };
    let reply_html = template.render().expect("Failed to render template");
    (StatusCode::OK, Html(reply_html).into_response())
}

pub fn serve_static_files() -> Router
{
    Router::new().fallback_service(ServeDir::new("public"))
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
