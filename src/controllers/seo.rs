use crate::{controllers::base::ControllerBase, pool::Db, services::BlogService};
use chrono::{DateTime, Utc};
use models::post;
use rocket::{
    fairing::{self, Fairing, Kind},
    http::Status,
    response::content,
    Build, Rocket, Route, State,
};
use sea_orm_rocket::Connection;

pub struct Controller {
    base: ControllerBase,
}

impl Controller {
    pub fn new(path: String) -> Self {
        Self {
            base: ControllerBase::new(path),
        }
    }
}

#[get("/sitemap.xml")]
async fn sitemap_xml(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
) -> Result<content::RawXml<String>, Status> {
    log::info!("Route accessed: GET /sitemap.xml - Sitemap requested");
    let db = conn.into_inner();
    
    // Fetch all published posts
    let posts = service
        .find_recent_published_posts(db, None) // Get all posts
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Generate sitemap XML
    let sitemap_xml = generate_sitemap_xml(&posts);
    
    Ok(content::RawXml(sitemap_xml))
}

fn generate_sitemap_xml(posts: &[post::Model]) -> String {
    let mut url_entries = String::new();
    
    // Add homepage
    url_entries.push_str(&format!(
        r#"
    <url>
        <loc>/</loc>
        <changefreq>daily</changefreq>
        <priority>1.0</priority>
    </url>"#
    ));
    
    // Add blog listing page
    url_entries.push_str(&format!(
        r#"
    <url>
        <loc>/blog</loc>
        <changefreq>daily</changefreq>
        <priority>0.9</priority>
    </url>"#
    ));
    
    // Add RSS feed
    url_entries.push_str(&format!(
        r#"
    <url>
        <loc>/feed/rss</loc>
        <changefreq>daily</changefreq>
        <priority>0.8</priority>
    </url>"#
    ));

    // Add individual blog posts
    for post in posts {
        let last_modified = DateTime::<Utc>::from_naive_utc_and_offset(post.date_published, Utc)
            .format("%Y-%m-%d");

        url_entries.push_str(&format!(
            r#"
    <url>
        <loc>/blog/{}</loc>
        <lastmod>{}</lastmod>
        <changefreq>weekly</changefreq>
        <priority>0.8</priority>
    </url>"#,
            post.seq_id, last_modified
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">{}
</urlset>"#,
        url_entries
    )
}

fn routes() -> Vec<Route> {
    routes![sitemap_xml]
}

crate::impl_controller_routes!(Controller, "SEO Controller", routes());