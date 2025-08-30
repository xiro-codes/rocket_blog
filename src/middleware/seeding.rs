use crate::pool::Db;
use chrono::{Local, Datelike};
use lipsum::lipsum_words_with_rng;
use models::{
    account, comment, post, post_tag, user_role, work_time_entry, pay_period,
    prelude::{Account, Comment, Post, PostTag, WorkTimeEntry},
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

const DATA_PATH: &str = "/home/tod/.local/share/blog";
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
        let pw = bcrypt::hash("pass").unwrap();
        let ac = account::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            username: Set("admin".to_owned()),
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

        // Create sample user roles for worktime tracking
        log::debug!("Creating sample user roles");
        let sample_roles = vec![
            ("Software Developer", 25.0, "USD", false),
            ("Freelance Consultant", 50.0, "USD", false),
            ("Restaurant Server", 15.0, "USD", true), // Tipped role
        ];

        let mut created_roles = Vec::new();
        for (role_name, hourly_wage, currency, is_tipped) in sample_roles {
            let role = user_role::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                account_id: Set(ac.id),
                role_name: Set(role_name.to_owned()),
                hourly_wage: Set(hourly_wage),
                currency: Set(currency.to_owned()),
                is_tipped: Set(is_tipped),
                is_active: Set(true),
                created_at: Set(Local::now().naive_local()),
                updated_at: Set(Local::now().naive_local()),
            }
            .insert(conn)
            .await
            .map_err(|e| {
                log::error!("Failed to seed user role '{}': {}", role_name, e);
                e
            })
            .expect("Failed to seed user role.");
            created_roles.push(role);
        }
        
        log::debug!("Created {} user roles", created_roles.len());

        // Create sample pay periods
        log::debug!("Creating sample pay periods");
        let current_date = Local::now().naive_local().date();
        
        // Create current pay period (2 weeks)
        let current_period_start = current_date - chrono::Duration::days(current_date.weekday().num_days_from_monday() as i64);
        let current_period_end = current_period_start + chrono::Duration::days(13);
        
        // Create previous pay period
        let previous_period_start = current_period_start - chrono::Duration::days(14);
        let previous_period_end = current_period_start - chrono::Duration::days(1);

        let sample_periods = vec![
            ("Previous Pay Period", previous_period_start, previous_period_end, false),
            ("Current Pay Period", current_period_start, current_period_end, true),
        ];

        let mut created_periods = Vec::new();
        for (period_name, start_date, end_date, is_active) in sample_periods {
            let period = pay_period::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                account_id: Set(ac.id),
                period_name: Set(period_name.to_owned()),
                start_date: Set(start_date),
                end_date: Set(end_date),
                is_active: Set(is_active),
                created_at: Set(Local::now().naive_local()),
                updated_at: Set(Local::now().naive_local()),
            }
            .insert(conn)
            .await
            .map_err(|e| {
                log::error!("Failed to seed pay period '{}': {}", period_name, e);
                e
            })
            .expect("Failed to seed pay period.");
            created_periods.push(period);
        }
        
        log::debug!("Created {} pay periods", created_periods.len());

        // Create sample work time entries
        log::debug!("Creating sample work time entries");
        let now = chrono::Utc::now();
        let mut work_entries = Vec::new();

        // Create some completed entries for the previous pay period
        for i in 0..5 {
            let days_ago = 20 - (i * 2); // Work entries from 20, 18, 16, 14, 12 days ago
            let start_time = now - chrono::Duration::days(days_ago);
            let end_time = start_time + chrono::Duration::hours(8); // 8-hour work days
            let duration = 8 * 60; // 8 hours in minutes

            let role_index = (i as usize) % created_roles.len();
            let role = &created_roles[role_index];

            let entry = work_time_entry::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                account_id: Set(ac.id),
                user_role_id: Set(role.id),
                pay_period_id: Set(Some(created_periods[0].id)), // Previous period
                start_time: Set(start_time),
                end_time: Set(Some(end_time)),
                duration: Set(Some(duration)),
                description: Set(Some(format!("Sample work day {} - {}", i + 1, role.role_name))),
                project: Set(if i % 2 == 0 { Some("Project Alpha".to_string()) } else { Some("Project Beta".to_string()) }),
                tips: Set(if role.is_tipped { Some(25.0 + (i as f64 * 5.0)) } else { None }),
                is_active: Set(false),
                created_at: Set(start_time),
                updated_at: Set(end_time),
            };
            work_entries.push(entry);
        }

        // Create some entries for the current pay period
        for i in 0..3 {
            let days_ago = 5 - (i * 2); // Work entries from 5, 3, 1 days ago
            let start_time = now - chrono::Duration::days(days_ago);
            let end_time = start_time + chrono::Duration::hours(6); // 6-hour work days
            let duration = 6 * 60; // 6 hours in minutes

            let role_index = (i as usize) % created_roles.len();
            let role = &created_roles[role_index];

            let entry = work_time_entry::ActiveModel {
                id: Set(uuid::Uuid::new_v4()),
                account_id: Set(ac.id),
                user_role_id: Set(role.id),
                pay_period_id: Set(Some(created_periods[1].id)), // Current period
                start_time: Set(start_time),
                end_time: Set(Some(end_time)),
                duration: Set(Some(duration)),
                description: Set(Some(format!("Current period work {} - {}", i + 1, role.role_name))),
                project: Set(if i % 2 == 0 { Some("Project Gamma".to_string()) } else { None }),
                tips: Set(if role.is_tipped { Some(15.0 + (i as f64 * 3.0)) } else { None }),
                is_active: Set(false),
                created_at: Set(start_time),
                updated_at: Set(end_time),
            };
            work_entries.push(entry);
        }

        log::debug!("Inserting {} work time entries into database", work_entries.len());
        WorkTimeEntry::insert_many(work_entries).exec(conn).await.map_err(|e| {
            log::error!("Failed to insert work time entries: {}", e);
            e
        }).unwrap();

        log::debug!("Worktime seeding completed - {} roles, {} pay periods, {} work entries created", 
                   created_roles.len(), created_periods.len(), 8);

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
