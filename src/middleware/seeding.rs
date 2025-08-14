use crate::pool::Db;
use chrono::Local;
use lipsum::lipsum_words_with_rng;
use models::account;
use models::comment;
use models::post;
use models::tag;
use models::post_tag;
use models::prelude::Account;
use models::prelude::Comment;
use models::prelude::Post;
use models::prelude::Tag;
use models::prelude::PostTag;
use pwhash::bcrypt;
use rand::thread_rng;
use rocket::{
    fairing::{self, Fairing, Kind},
    Build, Orbit, Rocket,
};
use sea_orm::*;
use sea_orm_rocket::Database;
use slug::slugify;

pub struct Seeding {
    count: usize,
    seed: Option<u32>
}
impl Seeding {
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
        
        // Create sample tags
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
            .expect("Failed to seed tag.");
            created_tags.push(tag);
        }
        
        let mut posts = Vec::new();
        let mut comments: Vec<comment::ActiveModel> = Vec::new();
        let mut post_tags: Vec<post_tag::ActiveModel> = Vec::new();
        
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
                    text: Set(lipsum_words_with_rng(text_rng, 1 + rand::random::<usize>() % 10)),
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
        PostTag::insert_many(post_tags).exec(conn).await.unwrap();
        Ok(rocket)
    }
    async fn on_shutdown(&self, rocket: &Rocket<Orbit>) {
        let conn = &Db::fetch(&rocket).unwrap().conn;
        
        // Clean up video files before deleting database records
        if let Ok(posts_with_videos) = Post::find()
            .filter(post::Column::Path.is_not_null())
            .all(conn)
            .await 
        {
            for post in posts_with_videos {
                if let Some(video_path) = post.path {
                    let _ = std::fs::remove_file(&video_path);
                }
            }
        }
        
        let _ = PostTag::delete_many().exec(conn).await;
        let _ = Tag::delete_many().exec(conn).await;
        let _ = Comment::delete_many().exec(conn).await;
        let _ = Post::delete_many().exec(conn).await;
        let _ = Account::delete_many().exec(conn).await;
    }
}
