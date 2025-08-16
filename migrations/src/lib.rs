pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_tables;
mod m20241201_000001_add_tags;
mod m20241202_000001_add_excerpt;
mod m20241203_000001_add_fulltext_search;
mod m20241204_000001_add_post_reactions;

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
        ]
    }
}
