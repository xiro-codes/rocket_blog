use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create poll table
        manager
            .create_table(
                Table::create()
                    .table(Poll::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Poll::Id).uuid().not_null().primary_key())
                    .col(
                        ColumnDef::new(Poll::SeqId)
                            .integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(Poll::Title).string().not_null())
                    .col(ColumnDef::new(Poll::Description).text())
                    .col(ColumnDef::new(Poll::AccountId).uuid().not_null())
                    .col(ColumnDef::new(Poll::DatePublished).date_time().not_null())
                    .col(
                        ColumnDef::new(Poll::Active)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_poll_account")
                            .from(Poll::Table, Poll::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create poll_option table
        manager
            .create_table(
                Table::create()
                    .table(PollOption::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PollOption::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PollOption::PollId).uuid().not_null())
                    .col(ColumnDef::new(PollOption::Text).string().not_null())
                    .col(
                        ColumnDef::new(PollOption::Position)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_poll_option_poll")
                            .from(PollOption::Table, PollOption::PollId)
                            .to(Poll::Table, Poll::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create poll_vote table
        manager
            .create_table(
                Table::create()
                    .table(PollVote::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PollVote::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PollVote::PollId).uuid().not_null())
                    .col(ColumnDef::new(PollVote::OptionId).uuid().not_null())
                    .col(ColumnDef::new(PollVote::IpAddress).string().not_null())
                    .col(ColumnDef::new(PollVote::SessionId).string())
                    .col(ColumnDef::new(PollVote::CreatedAt).date_time().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_poll_vote_poll")
                            .from(PollVote::Table, PollVote::PollId)
                            .to(Poll::Table, Poll::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_poll_vote_option")
                            .from(PollVote::Table, PollVote::OptionId)
                            .to(PollOption::Table, PollOption::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Unique constraint to prevent duplicate votes from same IP
                    .index(
                        Index::create()
                            .name("IDX_poll_vote_unique")
                            .col(PollVote::PollId)
                            .col(PollVote::IpAddress)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(PollVote::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(PollOption::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Poll::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Poll {
    Table,
    Id,
    SeqId,
    Title,
    Description,
    AccountId,
    DatePublished,
    Active,
}

#[derive(DeriveIden)]
enum PollOption {
    Table,
    Id,
    PollId,
    Text,
    Position,
}

#[derive(DeriveIden)]
enum PollVote {
    Table,
    Id,
    PollId,
    OptionId,
    IpAddress,
    SessionId,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
}