use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add YouTube URL and download status fields to post table
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

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
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
}

#[derive(DeriveIden)]
enum Post {
    Table,
    YoutubeUrl,
    DownloadStatus,
    DownloadError,
}