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
                    .col(
                        ColumnDef::new(Settings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Settings::Key)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Settings::Value).text().null())
                    .col(
                        ColumnDef::new(Settings::Description)
                            .string()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(Settings::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Settings::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Insert default OpenAI settings
        let insert = Query::insert()
            .into_table(Settings::Table)
            .columns([
                Settings::Id,
                Settings::Key,
                Settings::Value,
                Settings::Description,
            ])
            .values_panic([
                Expr::val("11111111-1111-1111-1111-111111111111").into(),
                "openai_api_key".into(),
                "".into(),
                "OpenAI API Key for post generation".into(),
            ])
            .values_panic([
                Expr::val("22222222-2222-2222-2222-222222222222").into(),
                "openai_model".into(),
                "gpt-3.5-turbo".into(),
                "OpenAI model to use for post generation".into(),
            ])
            .values_panic([
                Expr::val("33333333-3333-3333-3333-333333333333").into(),
                "openai_max_tokens".into(),
                "1000".into(),
                "Maximum tokens for OpenAI API responses".into(),
            ])
            .values_panic([
                Expr::val("44444444-4444-4444-4444-444444444444").into(),
                "openai_temperature".into(),
                "0.7".into(),
                "Temperature setting for OpenAI API (0.0-1.0)".into(),
            ])
            .to_owned();

        manager.exec_stmt(insert).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop settings table
        manager
            .drop_table(Table::drop().table(Settings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Settings {
    Table,
    Id,
    Key,
    Value,
    Description,
    CreatedAt,
    UpdatedAt,
}