use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create post_reaction table
        manager
            .create_table(
                Table::create()
                    .table(PostReaction::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PostReaction::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PostReaction::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostReaction::ReactionType).string().not_null())
                    .col(ColumnDef::new(PostReaction::IpAddress).string().not_null())
                    .col(ColumnDef::new(PostReaction::SessionId).string())
                    .col(ColumnDef::new(PostReaction::CreatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_post_reaction_post")
                            .from(PostReaction::Table, PostReaction::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Unique constraint to prevent duplicate reactions from same IP/session
                    .index(
                        Index::create()
                            .name("IDX_post_reaction_unique")
                            .col(PostReaction::PostId)
                            .col(PostReaction::ReactionType)
                            .col(PostReaction::IpAddress)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostReaction::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum PostReaction {
    Table,
    Id,
    PostId,
    ReactionType,
    IpAddress,
    SessionId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
}