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
