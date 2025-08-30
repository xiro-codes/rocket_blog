use sea_orm_migration::prelude::*;
use sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add YouTube URL and download status fields to post table
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL supports multiple alter options in one statement
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::YoutubeUrl).string().null())
                            .add_column(ColumnDef::new(Post::DownloadStatus).string().null())
                            .add_column(ColumnDef::new(Post::DownloadError).text().null())
                            .to_owned(),
                    )
                    .await
            },
            DatabaseBackend::Sqlite => {
                // SQLite requires separate alter statements for each column
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::YoutubeUrl).string().null())
                            .to_owned(),
                    )
                    .await?;
                    
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::DownloadStatus).string().null())
                            .to_owned(),
                    )
                    .await?;
                    
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .add_column(ColumnDef::new(Post::DownloadError).text().null())
                            .to_owned(),
                    )
                    .await
            },
            _ => {
                Err(DbErr::Custom("Unsupported database backend".to_string()))
            }
        }
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL supports multiple alter options in one statement
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .drop_column(Post::YoutubeUrl)
                            .drop_column(Post::DownloadStatus)
                            .drop_column(Post::DownloadError)
                            .to_owned(),
                    )
                    .await
            },
            DatabaseBackend::Sqlite => {
                // SQLite requires separate alter statements for each column
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .drop_column(Post::YoutubeUrl)
                            .to_owned(),
                    )
                    .await?;
                    
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .drop_column(Post::DownloadStatus)
                            .to_owned(),
                    )
                    .await?;
                    
                manager
                    .alter_table(
                        Table::alter()
                            .table(Post::Table)
                            .drop_column(Post::DownloadError)
                            .to_owned(),
                    )
                    .await
            },
            _ => {
                Err(DbErr::Custom("Unsupported database backend".to_string()))
            }
        }
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    YoutubeUrl,
    DownloadStatus,
    DownloadError,
}