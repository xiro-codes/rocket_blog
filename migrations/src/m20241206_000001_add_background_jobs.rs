use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BackgroundJob::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(BackgroundJob::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(BackgroundJob::JobType).string().not_null())
                    .col(ColumnDef::new(BackgroundJob::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(BackgroundJob::Payload).json())
                    .col(ColumnDef::new(BackgroundJob::Result).json())
                    .col(ColumnDef::new(BackgroundJob::Error).text())
                    .col(ColumnDef::new(BackgroundJob::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(BackgroundJob::UpdatedAt).date_time().not_null())
                    .col(ColumnDef::new(BackgroundJob::CompletedAt).date_time())
                    .col(ColumnDef::new(BackgroundJob::AccountId).uuid().not_null())
                    .index(Index::create().col(BackgroundJob::Status).col(BackgroundJob::CreatedAt))
                    .index(Index::create().col(BackgroundJob::AccountId))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BackgroundJob::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum BackgroundJob {
    Table,
    Id,
    JobType,
    Status,
    Payload,
    Result,
    Error,
    CreatedAt,
    UpdatedAt,
    CompletedAt,
    AccountId,
}