use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create settings table
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Settings::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Settings::OpenaiApiKey).string())
                    .col(ColumnDef::new(Settings::OpenaiBasePrompt).text())
                    .col(ColumnDef::new(Settings::CreatedAt).date_time().not_null())
                    .col(ColumnDef::new(Settings::UpdatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Settings {
    Table,
    Id,
    OpenaiApiKey,
    OpenaiBasePrompt,
    CreatedAt,
    UpdatedAt,
}