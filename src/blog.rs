use chrono::NaiveDate;
use once_cell::sync::Lazy;
use pulldown_cmark::{html, Options, Parser};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::RwLock;

#[derive(Clone, Debug)]
pub struct Post
{
    pub slug: String,
    pub title: String,
    pub date: NaiveDate,
    pub tags: Vec<String>,
    pub summary: String,
    pub html_content: String,
}

#[derive(Deserialize)]
struct Frontmatter
{
    title: String,
    date: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    summary: Option<String>,
}

static POST_CACHE: Lazy<RwLock<Vec<Post>>> = Lazy::new(|| RwLock::new(Vec::new()));

pub fn load_posts()
{
    let posts_dir = Path::new("posts");
    if !posts_dir.exists() {
        tracing::warn!("posts/ directory not found, no blog posts loaded");
        return;
    }

    let mut posts = Vec::new();

    let mut entries: Vec<_> = fs::read_dir(posts_dir)
        .expect("Failed to read posts directory")
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "md")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());

    for entry in entries {
        let path = entry.path();
        match parse_post(&path) {
            Ok(post) => {
                tracing::info!("Loaded post: {}", post.title);
                posts.push(post);
            }
            Err(e) => {
                tracing::error!("Failed to parse {:?}: {}", path, e);
            }
        }
    }

    posts.sort_by(|a, b| b.date.cmp(&a.date));

    let mut cache = POST_CACHE.write().unwrap();
    *cache = posts;
}

fn parse_post(path: &Path) -> Result<Post, String>
{
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    let (frontmatter_str, markdown_body) = split_frontmatter(&content)?;
    let frontmatter: Frontmatter =
        serde_yaml::from_str(&frontmatter_str).map_err(|e| format!("YAML parse error: {}", e))?;

    let date =
        NaiveDate::parse_from_str(&frontmatter.date, "%Y-%m-%d").map_err(|e| e.to_string())?;

    let html_content = render_markdown(&markdown_body);

    let slug = path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();

    let summary = frontmatter.summary.unwrap_or_else(|| {
        let plain: String = markdown_body.chars().take(200).collect();
        if plain.len() >= 200 {
            format!("{}...", plain.trim())
        } else {
            plain.trim().to_string()
        }
    });

    Ok(Post {
        slug,
        title: frontmatter.title,
        date,
        tags: frontmatter.tags,
        summary,
        html_content,
    })
}

fn split_frontmatter(content: &str) -> Result<(String, String), String>
{
    let content = content.trim_start();
    if !content.starts_with("---") {
        return Err("Missing frontmatter delimiter".to_string());
    }

    let after_first = &content[3..];
    let end = after_first
        .find("\n---")
        .ok_or("Missing closing frontmatter delimiter")?;

    let frontmatter = after_first[..end].trim().to_string();
    let body = after_first[end + 4..].trim().to_string();

    Ok((frontmatter, body))
}

fn render_markdown(markdown: &str) -> String
{
    let options = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

pub fn get_all_posts() -> Vec<Post>
{
    POST_CACHE.read().unwrap().clone()
}

pub fn get_post_by_slug(slug: &str) -> Option<Post>
{
    POST_CACHE
        .read()
        .unwrap()
        .iter()
        .find(|p| p.slug == slug)
        .cloned()
}

pub fn get_all_tags() -> Vec<String>
{
    let posts = POST_CACHE.read().unwrap();
    let mut tag_counts: HashMap<String, usize> = HashMap::new();
    for post in posts.iter() {
        for tag in &post.tags {
            *tag_counts.entry(tag.clone()).or_insert(0) += 1;
        }
    }
    let mut tags: Vec<_> = tag_counts.into_iter().collect();
    tags.sort_by(|a, b| b.1.cmp(&a.1));
    tags.into_iter().map(|(tag, _)| tag).collect()
}
