use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create tags table
        manager
            .create_table(
                Table::create()
                    .table(Tag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Tag::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Tag::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Tag::Slug).string().not_null().unique_key())
                    .col(ColumnDef::new(Tag::Color).string().default("#007bff"))
                    .col(ColumnDef::new(Tag::CreatedAt).date_time().not_null())
                    .to_owned(),
            )
            .await?;

        // Create post_tags junction table for many-to-many relationship
        manager
            .create_table(
                Table::create()
                    .table(PostTag::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PostTag::PostId).uuid().not_null())
                    .col(ColumnDef::new(PostTag::TagId).uuid().not_null())
                    .primary_key(
                        Index::create()
                            .col(PostTag::PostId)
                            .col(PostTag::TagId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_post_tag_post")
                            .from(PostTag::Table, PostTag::PostId)
                            .to(Post::Table, Post::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_post_tag_tag")
                            .from(PostTag::Table, PostTag::TagId)
                            .to(Tag::Table, Tag::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PostTag::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Tag::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Tag {
    Table,
    Id,
    Name,
    Slug,
    Color,
    CreatedAt,
}

#[derive(DeriveIden)]
enum PostTag {
    Table,
    PostId,
    TagId,
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
}