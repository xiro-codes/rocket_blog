use sea_orm_migration::prelude::*;
use super::m20220101_000001_create_tables::Account;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(
            Table::create()
                .table(Event::Table)
                .if_not_exists()
                .col(ColumnDef::new(Event::Id).uuid().not_null().primary_key())
                .col(ColumnDef::new(Event::OwnerId).uuid().not_null())
                .col(ColumnDef::new(Event::Title).string().not_null())
                .col(ColumnDef::new(Event::StartDate).date_time().not_null())
                .col(ColumnDef::new(Event::EndDate).date_time().not_null())

                .foreign_key(
                    ForeignKey::create()
                        .name("FK_event_account")
                        .from(Event::Table, Event::OwnerId)
                        .to(Account::Table, Account::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade),
                ).to_owned()
        ).await?;
        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Event::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Event {
    Table,
    Id,
    OwnerId,
    Title,
    StartDate,
    EndDate,
}
