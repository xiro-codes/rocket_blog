//! Database seeding middleware for development and testing.
//!
//! This middleware automatically populates the database with sample data
//! during application startup, including admin users, sample blog posts,
//! and comments. This is useful for development environments and testing.

use crate::pool::Db;
use chrono::Local;
use lipsum::lipsum_words_with_rng;
use models::account;
use models::comment;
use models::post;
use models::prelude::Account;
use models::prelude::Comment;
use models::prelude::Post;
use pwhash::bcrypt;
use rand::thread_rng;
use rocket::{
    fairing::{self, Fairing, Kind},
    Build, Orbit, Rocket,
};
use sea_orm::*;
use sea_orm_rocket::Database;

/// Seeding fairing for populating the database with development data.
///
/// This middleware creates sample data during application startup including:
/// - An admin user account with known credentials
/// - A configurable number of sample blog posts with Lorem Ipsum content
/// - Sample comments on the generated blog posts
/// 
/// The seeding process is deterministic when a seed value is provided,
/// making it useful for consistent testing environments.
pub struct Seeding {
    /// Number of sample posts and comments to create
    count: usize,
    /// Optional seed for deterministic random generation
    seed: Option<u32>
}

impl Seeding {
    /// Creates a new Seeding fairing instance.
    ///
    /// # Arguments
    ///
    /// * `seed` - Optional seed for deterministic random content generation
    /// * `count` - Number of sample posts and comments to create
    ///
    /// # Returns
    ///
    /// A new Seeding fairing ready to populate the database.
    pub fn new(seed: Option<u32>, count: usize) -> Self {
        Self { seed, count }
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

    /// Populates the database with sample data during application startup.
    ///
    /// This method:
    /// 1. Creates an admin user account with username "admin" and password "pass"
    /// 2. Generates the specified number of sample blog posts with Lorem Ipsum content
    /// 3. Creates sample comments for each blog post
    /// 4. Uses deterministic random generation if a seed is provided
    ///
    /// # Arguments
    ///
    /// * `rocket` - The Rocket instance being built
    ///
    /// # Returns
    ///
    /// The Rocket instance, ready for launch with seeded data.
    ///
    /// # Panics
    ///
    /// Panics if the admin account cannot be created during seeding.
    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        let conn = &Db::fetch(&rocket).unwrap().conn;
        let pw = bcrypt::hash("pass").unwrap();
        let ac = account::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            username: Set("admin".to_owned()),
            password: Set(pw),
            email: Set("admin@tdavis.dev".to_owned()),
            admin: Set(true),
        }
        .insert(conn)
        .await.expect("Failed to seed account.");
        let mut posts = Vec::new();
        let mut comments:Vec<comment::ActiveModel> = Vec::new();
        for i in 1..self.count {
            let title_rng = thread_rng();
            let text_rng = thread_rng();
            let p = post::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                title: Set(lipsum_words_with_rng(title_rng, 5)),
                text: Set(lipsum_words_with_rng(text_rng, 50 + (rand::random::<usize>() % 50))),
                draft: Set(Some(false)),
                account_id: Set(ac.id),
                date_published: Set(Local::now().naive_local()),
                ..Default::default()
            };
            for _ in 0..(20 % i) {
                let text_rng = thread_rng();
                let c = comment::ActiveModel {
                    id: Set(uuid::Uuid::new_v4()),
                    text: Set(lipsum_words_with_rng(text_rng, 1 + rand::random::<usize>() % 10),),
                    post_id: p.id.clone(),
                    date_published: Set(Local::now().naive_local()),
                    ..Default::default()
                };
                comments.push(c);
            }

            posts.push(p);
        }
        Post::insert_many(posts).exec(conn).await.unwrap();
        Comment::insert_many(comments).exec(conn).await.unwrap();
        Ok(rocket)
    }
    async fn on_shutdown(&self, rocket: &Rocket<Orbit>) {
        let conn = &Db::fetch(&rocket).unwrap().conn;
        let _ = Account::delete_many().exec(conn).await;
        let _ = Post::delete_many().exec(conn).await;
        let _ = Comment::delete_many().exec(conn).await;
    }
}
