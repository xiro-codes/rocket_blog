use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add is_tipped column to user_role table
        manager
            .alter_table(
                Table::alter()
                    .table(UserRole::Table)
                    .add_column(ColumnDef::new(UserRole::IsTipped).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove is_tipped column from user_role table
        manager
            .alter_table(
                Table::alter()
                    .table(UserRole::Table)
                    .drop_column(UserRole::IsTipped)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum UserRole {
    Table,
    IsTipped,
}