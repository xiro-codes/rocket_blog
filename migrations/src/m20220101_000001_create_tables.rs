use sea_orm_migration::prelude::*;
use sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Detect database backend for auto_increment compatibility
        let db_backend = manager.get_database_backend();
        
        manager
            .create_table(
                Table::create()
                    .table(Account::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Account::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Account::Username).string().not_null())
                    .col(ColumnDef::new(Account::Email).string().not_null())
                    .col(ColumnDef::new(Account::Password).string().not_null())
                    .col(
                        ColumnDef::new(Account::Admin)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .to_owned(),
            )
            .await?;
            
        // Create Post table with database-specific auto_increment handling
        let mut post_table = Table::create()
            .table(Post::Table)
            .if_not_exists()
            .col(ColumnDef::new(Post::Id).uuid().not_null().primary_key())
            .to_owned();
            
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL supports auto_increment on non-primary key columns
                post_table.col(
                    ColumnDef::new(Post::SeqId)
                        .integer()
                        .not_null()
                        .auto_increment(),
                );
            },
            DatabaseBackend::Sqlite => {
                // SQLite: Use a regular integer, will be managed by application or triggers
                post_table.col(
                    ColumnDef::new(Post::SeqId)
                        .integer()
                        .not_null(),
                );
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }
        
        post_table
            .col(ColumnDef::new(Post::Title).string().not_null())
            .col(ColumnDef::new(Post::Text).string().not_null())
            .col(ColumnDef::new(Post::Path).string().default(""))
            .col(ColumnDef::new(Post::Draft).boolean().default(false))
            .col(ColumnDef::new(Post::DatePublished).date_time().not_null())
            .col(ColumnDef::new(Post::AccountId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .name("FK_post_account")
                    .from(Post::Table, Post::AccountId)
                    .to(Account::Table, Account::Id)
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::Cascade),
            );
            
        manager.create_table(post_table).await?;

        manager
            .create_table(
                Table::create()
                    .table(Comment::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Comment::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Comment::PostId).uuid().not_null())
                    .col(ColumnDef::new(Comment::Text).string().not_null())
                    .col(
                        ColumnDef::new(Comment::DatePublished)
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_comment_post")
                            .from(Comment::Table, Comment::PostId)
                            .to(Post::Table, Post::Id)
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
            .drop_table(Table::drop().table(Comment::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Account::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
    Username,
    Password,
    Email,
    Admin,
}

#[derive(DeriveIden)]
enum Post {
    Table,
    Id,
    SeqId,
    AccountId,
    Title,
    Text,
    Path,
    Draft,
    DatePublished,
}

#[derive(DeriveIden)]
enum Comment {
    Table,
    Id,
    PostId,
    Text,
    DatePublished,
}
