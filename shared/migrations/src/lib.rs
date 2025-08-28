pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20241201_000001_add_tags;
mod m20241202_000001_add_excerpt;
mod m20241203_000001_add_fulltext_search;
mod m20241204_000001_add_post_reactions;
mod m20241205_000001_add_settings;
mod m20241206_000001_add_comment_user_association;
mod m20241207_000001_add_comment_replies;
mod m20241210_000001_add_youtube_downloads;
mod m20241212_000001_add_background_jobs;
mod m20241213_000001_add_punch_clock;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_tables::Migration),
            Box::new(m20241201_000001_add_tags::Migration),
            Box::new(m20241202_000001_add_excerpt::Migration),
            Box::new(m20241203_000001_add_fulltext_search::Migration),
            Box::new(m20241204_000001_add_post_reactions::Migration),
            Box::new(m20241205_000001_add_settings::Migration),
            Box::new(m20241206_000001_add_comment_user_association::Migration),
            Box::new(m20241207_000001_add_comment_replies::Migration),
            Box::new(m20241210_000001_add_youtube_downloads::Migration),
            Box::new(m20241212_000001_add_background_jobs::Migration),
            Box::new(m20241213_000001_add_punch_clock::Migration),
        ]
    }
}
