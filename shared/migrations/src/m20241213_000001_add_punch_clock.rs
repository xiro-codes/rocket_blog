use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create work_role table
        manager
            .create_table(
                Table::create()
                    .table(WorkRole::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(WorkRole::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WorkRole::Name).string().not_null())
                    .col(ColumnDef::new(WorkRole::HourlyRate).decimal_len(10, 2).not_null())
                    .col(ColumnDef::new(WorkRole::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(WorkRole::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(WorkRole::UpdatedAt).timestamp().not_null())
                    .to_owned(),
            )
            .await?;

        // Create work_session table
        manager
            .create_table(
                Table::create()
                    .table(WorkSession::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(WorkSession::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WorkSession::AccountId).uuid().not_null())
                    .col(ColumnDef::new(WorkSession::WorkRoleId).uuid().not_null())
                    .col(ColumnDef::new(WorkSession::ClockInTime).timestamp().not_null())
                    .col(ColumnDef::new(WorkSession::ClockOutTime).timestamp())
                    .col(ColumnDef::new(WorkSession::DurationMinutes).integer())
                    .col(ColumnDef::new(WorkSession::Earnings).decimal_len(10, 2))
                    .col(ColumnDef::new(WorkSession::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(WorkSession::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_work_session_account")
                            .from(WorkSession::Table, WorkSession::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_work_session_work_role")
                            .from(WorkSession::Table, WorkSession::WorkRoleId)
                            .to(WorkRole::Table, WorkRole::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create index for active sessions
        manager
            .create_index(
                Index::create()
                    .name("idx_work_session_active")
                    .table(WorkSession::Table)
                    .col(WorkSession::AccountId)
                    .col(WorkSession::ClockOutTime)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkSession::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(WorkRole::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Account {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum WorkRole {
    Table,
    Id,
    Name,
    HourlyRate,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WorkSession {
    Table,
    Id,
    AccountId,
    WorkRoleId,
    ClockInTime,
    ClockOutTime,
    DurationMinutes,
    Earnings,
    CreatedAt,
    UpdatedAt,
}