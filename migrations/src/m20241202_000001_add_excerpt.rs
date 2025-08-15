use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add excerpt column to the post table
        manager
            .alter_table(
                Table::alter()
                    .table(Post::Table)
                    .add_column(ColumnDef::new(Post::Excerpt).string().null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove excerpt column from the post table
        manager
            .alter_table(
                Table::alter()
                    .table(Post::Table)
                    .drop_column(Post::Excerpt)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Excerpt,
}