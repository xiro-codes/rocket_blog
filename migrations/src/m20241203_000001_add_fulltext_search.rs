use sea_orm_migration::prelude::*;
use sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Detect database backend
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL: Add tsvector column for full-text search
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::SearchVector).custom(Alias::new("tsvector")).null())
                            .to_owned(),
                    )
                    .await?;

                // Create a GIN index on the search vector column for performance using raw SQL
                let gin_index_sql = r#"
                CREATE INDEX IF NOT EXISTS idx_post_search_vector ON post USING gin(search_vector);
                "#;
                manager.get_connection().execute_unprepared(gin_index_sql).await?;

                // Create a trigger function to automatically update the search vector
                let sql = r#"
                CREATE OR REPLACE FUNCTION update_post_search_vector() RETURNS trigger AS $$
                BEGIN
                    NEW.search_vector := to_tsvector('english', 
                        COALESCE(NEW.title, '') || ' ' ||
                        COALESCE(NEW.text, '') || ' ' ||
                        COALESCE(NEW.excerpt, '')
                    );
                    RETURN NEW;
                END;
                $$ LANGUAGE plpgsql;
                "#;
                manager.get_connection().execute_unprepared(sql).await?;

                // Create trigger to update search vector on insert/update
                let trigger_sql = r#"
                CREATE TRIGGER post_search_vector_update
                BEFORE INSERT OR UPDATE ON post
                FOR EACH ROW EXECUTE FUNCTION update_post_search_vector();
                "#;
                manager.get_connection().execute_unprepared(trigger_sql).await?;

                // Update existing posts to populate search vector
                let update_sql = r#"
                UPDATE post SET search_vector = to_tsvector('english', 
                    COALESCE(title, '') || ' ' ||
                    COALESCE(text, '') || ' ' ||
                    COALESCE(excerpt, '')
                );
                "#;
                manager.get_connection().execute_unprepared(update_sql).await?;
            },
            DatabaseBackend::Sqlite => {
                // SQLite: Add a simple text column for search content
                // SQLite has built-in FTS, but we'll use a simpler approach for compatibility
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::SearchVector).text().null())
                            .to_owned(),
                    )
                    .await?;

                // Create a simple index on the search vector column
                manager
                    .create_index(
                        Index::create()
                            .name("idx_post_search_vector")
                            .table(Post::Table)
                            .col(Post::SearchVector)
                            .to_owned(),
                    )
                    .await?;

                // Update existing posts to populate search vector with concatenated text
                let update_sql = r#"
                UPDATE post SET search_vector = 
                    COALESCE(title, '') || ' ' ||
                    COALESCE(text, '') || ' ' ||
                    COALESCE(excerpt, '');
                "#;
                manager.get_connection().execute_unprepared(update_sql).await?;
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Detect database backend
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // Drop trigger
                manager
                    .get_connection()
                    .execute_unprepared("DROP TRIGGER IF EXISTS post_search_vector_update ON post;")
                    .await?;

                // Drop trigger function
                manager
                    .get_connection()
                    .execute_unprepared("DROP FUNCTION IF EXISTS update_post_search_vector();")
                    .await?;

                // Drop index using raw SQL
                manager
                    .get_connection()
                    .execute_unprepared("DROP INDEX IF EXISTS idx_post_search_vector;")
                    .await?;
            },
            DatabaseBackend::Sqlite => {
                // Drop the index
                manager
                    .drop_index(
                        Index::drop()
                            .name("idx_post_search_vector")
                            .to_owned(),
                    )
                    .await?;
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        // Drop search vector column (common for both databases)
        manager
            .alter_table(
                Table::alter()
                    .table(Post::Table)
                    .drop_column(Post::SearchVector)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    SearchVector,
}