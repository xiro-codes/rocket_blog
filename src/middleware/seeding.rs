use crate::pool::Db;
use chrono::Local;
use lipsum::lipsum_words_with_rng;
use models::{
    account, comment, post, post_tag,
    prelude::{Account, Comment, Post, PostTag, Tag},
    tag,
};
use pwhash::bcrypt;
use rand::thread_rng;
use rocket::{
    fairing::{self, Fairing, Kind},
    Build, Orbit, Rocket,
};
use sea_orm::*;
use sea_orm_rocket::Database;
use slug::slugify;
use std::path::Path;

pub struct Seeding {
    count: usize,
    seed: Option<u32>,
}

const SAMPLE_VIDEO_PATH: &str = "static/sample_video.webm";

impl Seeding {
    pub fn new(seed: Option<u32>, count: usize) -> Self {
        Self { seed, count }
    }

    // Helper function to get the sample video path if it exists
    fn get_sample_video_path(&self) -> Option<String> {
        if Path::new(SAMPLE_VIDEO_PATH).exists() {
            Some(SAMPLE_VIDEO_PATH.to_string())
        } else {
            println!("Warning: Sample video file not found at {}. Video posts will be created without video files.", SAMPLE_VIDEO_PATH);
            None
        }
    }
}
#[rocket::async_trait]
impl Fairing for Seeding {
    fn info(&self) -> fairing::Info {
        fairing::Info {
            name: "Seeding",
            kind: Kind::Ignite | Kind::Shutdown,
        }
    }
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        log::info!("Seeding middleware: checking if database seeding is needed");
        let conn = &Db::fetch(&rocket).unwrap().conn;
        
        // Check if data already exists - only seed if database is empty
        let account_count = Account::find().count(conn).await.unwrap_or(0);
        if account_count > 0 {
            log::info!("Database already contains {} accounts, skipping seeding", account_count);
            return Ok(rocket);
        }
        
        log::info!("Database is empty, creating seed data with {} posts...", self.count);
        
        log::debug!("Creating admin account");
        let admin_username = std::env::var("DEFAULT_ADMIN_USERNAME").unwrap_or_else(|_| "admin".to_owned());
        let admin_password = std::env::var("DEFAULT_ADMIN_PASSWORD").unwrap_or_else(|_| "pass".to_owned());
        let pw = bcrypt::hash(admin_password).unwrap();
        let ac = account::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            username: Set(admin_username),
            password: Set(pw),
            email: Set("admin@tdavis.dev".to_owned()),
            admin: Set(true),
        }
        .insert(conn)
        .await
        .map_err(|e| {
            log::error!("Failed to seed admin account: {}", e);
            e
        })
        .expect("Failed to seed account.");
        
        log::debug!("Admin account created: {} ({})", ac.username, ac.id);

        if self.count <= 1 {
            log::info!("Database seeding completed successfully (admin only)");
            return Ok(rocket);
        }

        // Create sample tags
        log::debug!("Creating sample tags");
        let sample_tags = vec![
            ("Rust", "#CE422B"),
            ("Web Development", "#61DAFB"),
            ("Tutorial", "#28A745"),
            ("Programming", "#6F42C1"),
            ("Backend", "#FD7E14"),
            ("Database", "#17A2B8"),
            ("Framework", "#DC3545"),
        ];

        let mut created_tags = Vec::new();
        for (tag_name, color) in sample_tags {
            let tag = tag::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                name: Set(tag_name.to_owned()),
                slug: Set(slugify(tag_name)),
                color: Set(Some(color.to_owned())),
                created_at: Set(Local::now().naive_local()),
            }
            .insert(conn)
            .await
            .map_err(|e| {
                log::error!("Failed to seed tag '{}': {}", tag_name, e);
                e
            })
            .expect("Failed to seed tag.");
            created_tags.push(tag);
        }
        
        log::debug!("Created {} tags", created_tags.len());

        // Get the sample video path if it exists
        let video_path = self.get_sample_video_path();
        if video_path.is_some() {
            log::debug!("Sample video found, will add to some posts");
        } else {
            log::debug!("No sample video found, posts will be text-only");
        }

        let mut posts = Vec::new();
        let mut comments: Vec<comment::ActiveModel> = Vec::new();
        let mut post_tags: Vec<post_tag::ActiveModel> = Vec::new();

        log::debug!("Creating {} sample posts with comments and tags", self.count);
        for i in 1..self.count {
            let title_rng = thread_rng();
            let text_rng = thread_rng();
            let excerpt_rng = thread_rng();

            // Assign video path to about 30% of posts if sample video exists
            let post_video_path = if video_path.is_some() && rand::random::<f32>() < 0.75 {
                video_path.clone()
            } else {
                None
            };

            // Add custom excerpts to about 60% of posts, leave others for auto-generation
            let post_excerpt = if rand::random::<f32>() < 0.6 {
                Some(lipsum_words_with_rng(excerpt_rng, 15 + (rand::random::<usize>() % 10)))
            } else {
                None
            };

            let p = post::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                seq_id: Set(i as i32),
                title: Set(lipsum_words_with_rng(title_rng, 5)),
                text: Set(lipsum_words_with_rng(
                    text_rng,
                    50 + (rand::random::<usize>() % 50),
                )),
                excerpt: Set(post_excerpt),
                path: Set(post_video_path),
                draft: Set(Some(false)),
                account_id: Set(ac.id),
                date_published: Set(Local::now().naive_local()),
                ..Default::default()
            };

            // Add 1-3 random tags to each post
            let num_tags = 1 + (rand::random::<usize>() % 3);
            let mut used_tags = std::collections::HashSet::new();
            for _ in 0..num_tags {
                let mut tag_index = rand::random::<usize>() % created_tags.len();
                while used_tags.contains(&tag_index) && used_tags.len() < created_tags.len() {
                    tag_index = rand::random::<usize>() % created_tags.len();
                }
                used_tags.insert(tag_index);

                let pt = post_tag::ActiveModel {
                    post_id: p.id.clone(),
                    tag_id: Set(created_tags[tag_index].id),
                };
                post_tags.push(pt);
            }

            for _ in 0..(20 % i) {
                let text_rng = thread_rng();
                let c = comment::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    text: Set(lipsum_words_with_rng(
                        text_rng,
                        1 + rand::random::<usize>() % 10,
                    )),
                    post_id: p.id.clone(),
                    date_published: Set(Local::now().naive_local()),
                    ..Default::default()
                };
                comments.push(c);
            }

            posts.push(p);
        }

        log::debug!("Inserting {} posts into database", posts.len());
        Post::insert_many(posts).exec(conn).await.map_err(|e| {
            log::error!("Failed to insert posts: {}", e);
            e
        }).unwrap();
        
        log::debug!("Inserting {} comments into database", comments.len());
        Comment::insert_many(comments).exec(conn).await.map_err(|e| {
            log::error!("Failed to insert comments: {}", e);
            e
        }).unwrap();
        
        log::debug!("Inserting {} post-tag relationships into database", post_tags.len());
        PostTag::insert_many(post_tags).exec(conn).await.map_err(|e| {
            log::error!("Failed to insert post-tag relationships: {}", e);
            e
        }).unwrap();
        
        log::info!("Database seeding completed successfully");
        Ok(rocket)
    }
    async fn on_shutdown(&self, _rocket: &Rocket<Orbit>) {
        // Data persistence: Do not delete data on shutdown to preserve 
        // database content across container restarts
        log::info!("Application shutting down - preserving database data");
    }
}
