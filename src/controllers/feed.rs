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

#[get("/rss")]
async fn rss_feed(
    conn: Connection<'_, Db>,
    service: &State<BlogService>,
) -> Result<content::RawXml<String>, Status> {
    log::info!("Route accessed: GET /feed/rss - RSS feed requested");
    let db = conn.into_inner();
    
    // Fetch recent published posts
    let posts = service
        .find_recent_published_posts(db, Some(20))
        .await
        .map_err(|_| Status::InternalServerError)?;

    // Generate RSS XML
    let rss_xml = generate_rss_xml(&posts);
    
    Ok(content::RawXml(rss_xml))
}

fn generate_rss_xml(posts: &[post::Model]) -> String {
    let now = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT");
    
    let mut items = String::new();
    for post in posts {
        let pub_date = DateTime::<Utc>::from_naive_utc_and_offset(post.date_published, Utc)
            .format("%a, %d %b %Y %H:%M:%S GMT");
        
        let description = post.excerpt
            .as_ref()
            .unwrap_or(&post.text)
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;");

        let title = post.title
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;");

        items.push_str(&format!(
            r#"
        <item>
            <title>{}</title>
            <link>http://localhost:8000/blog/{}</link>
            <description>{}</description>
            <pubDate>{}</pubDate>
            <guid>http://localhost:8000/blog/{}</guid>
        </item>"#,
            title, post.seq_id, description, pub_date, post.seq_id
        ));
    }

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>Rocket Blog</title>
        <link>http://localhost:8000</link>
        <description>A blog built with Rocket framework</description>
        <language>en-us</language>
        <lastBuildDate>{}</lastBuildDate>
        <generator>Rocket Blog RSS Generator</generator>{}
    </channel>
</rss>"#,
        now, items
    )
}

fn routes() -> Vec<Route> {
    routes![rss_feed]
}

#[rocket::async_trait]
impl Fairing for Controller {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Feed Controller",
            kind: Kind::Ignite,
        }
    }
    
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        Ok(rocket.mount(self.base.path(), routes()))
    }
}