use askama_axum::Template;

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