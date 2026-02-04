use askama::Template;

use crate::blog::Post;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a>
{
    pub title: &'a str,
}

#[derive(Template)]
#[template(path = "error.html")]
pub struct ErrorTemplate
{
    pub status: u16,
    pub message: String,
}

#[derive(Template)]
#[template(path = "blog.html")]
pub struct BlogTemplate<'a>
{
    pub title: &'a str,
    pub posts: Vec<Post>,
    pub tags: Vec<String>,
    pub active_tag: Option<String>,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a>
{
    pub title: &'a str,
    pub post: Post,
}

#[derive(Template)]
#[template(path = "latest_posts.html")]
pub struct LatestPostsTemplate
{
    pub posts: Vec<Post>,
}
