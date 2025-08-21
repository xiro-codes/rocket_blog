use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create background_jobs table for generic job processing
        manager
            .create_table(
                Table::create()
                    .table(BackgroundJob::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BackgroundJob::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::JobType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::EntityType)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::EntityId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::Status)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::JobData)
                            .json()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(BackgroundJob::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for faster lookups
        manager
            .create_index(
                Index::create()
                    .name("idx_background_jobs_entity")
                    .table(BackgroundJob::Table)
                    .col(BackgroundJob::EntityType)
                    .col(BackgroundJob::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_background_jobs_status")
                    .table(BackgroundJob::Table)
                    .col(BackgroundJob::Status)
                    .to_owned(),
            )
            .await?;

        // Remove YouTube-specific columns from post table
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
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop background_jobs table
        manager
            .drop_table(Table::drop().table(BackgroundJob::Table).to_owned())
            .await?;

        // Re-add YouTube columns to post table
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
    }
}

#[derive(DeriveIden)]
enum BackgroundJob {
    Table,
    Id,
    JobType,
    EntityType,
    EntityId,
    Status,
    ErrorMessage,
    JobData,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Post {
    Table,
    YoutubeUrl,
    DownloadStatus,
    DownloadError,
}