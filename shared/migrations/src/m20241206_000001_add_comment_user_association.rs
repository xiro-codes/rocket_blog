use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add user_id column to comments table (nullable, for authenticated users)
        manager
            .alter_table(
                Table::alter()
                    .table(Comment::Table)
                    .add_column(ColumnDef::new(Comment::UserId).uuid().null())
                    .to_owned(),
            )
            .await?;

        // Add username column to comments table (nullable, for anonymous users) 
        manager
            .alter_table(
                Table::alter()
                    .table(Comment::Table)
                    .add_column(ColumnDef::new(Comment::Username).string().null())
                    .to_owned(),
            )
            .await?;

        // For existing comments, set username to "Anonymous" to handle the requirement
        // "if a comment already exists assign it to a new anon user"
        manager
            .get_connection()
            .execute_unprepared("UPDATE comment SET username = 'Anonymous' WHERE username IS NULL")
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop columns
        manager
            .alter_table(
                Table::alter()
                    .table(Comment::Table)
                    .drop_column(Comment::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Comment::Table)
                    .drop_column(Comment::Username)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Comment {
    Table,
    UserId,
    Username,
}