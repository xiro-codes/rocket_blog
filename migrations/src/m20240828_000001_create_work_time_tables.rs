use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_roles table for role-based wage management
        manager
            .create_table(
                Table::create()
                    .table(UserRole::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UserRole::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(UserRole::AccountId).uuid().not_null())
                    .col(ColumnDef::new(UserRole::RoleName).string().not_null())
                    .col(ColumnDef::new(UserRole::HourlyWage).decimal().not_null())
                    .col(ColumnDef::new(UserRole::Currency).string().not_null().default("USD"))
                    .col(ColumnDef::new(UserRole::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(UserRole::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(UserRole::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_user_role_account")
                            .from(UserRole::Table, UserRole::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create work_time_entries table for logging work times
        manager
            .create_table(
                Table::create()
                    .table(WorkTimeEntry::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(WorkTimeEntry::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(WorkTimeEntry::AccountId).uuid().not_null())
                    .col(ColumnDef::new(WorkTimeEntry::UserRoleId).uuid().not_null())
                    .col(ColumnDef::new(WorkTimeEntry::StartTime).timestamp().not_null())
                    .col(ColumnDef::new(WorkTimeEntry::EndTime).timestamp())
                    .col(ColumnDef::new(WorkTimeEntry::Duration).integer()) // Duration in minutes
                    .col(ColumnDef::new(WorkTimeEntry::Description).string())
                    .col(ColumnDef::new(WorkTimeEntry::Project).string())
                    .col(ColumnDef::new(WorkTimeEntry::IsActive).boolean().not_null().default(false)) // For active time tracking
                    .col(ColumnDef::new(WorkTimeEntry::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(WorkTimeEntry::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_work_time_entry_account")
                            .from(WorkTimeEntry::Table, WorkTimeEntry::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_work_time_entry_user_role")
                            .from(WorkTimeEntry::Table, WorkTimeEntry::UserRoleId)
                            .to(UserRole::Table, UserRole::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for better performance
        manager
            .create_index(
                Index::create()
                    .name("idx_user_role_account_id")
                    .table(UserRole::Table)
                    .col(UserRole::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_work_time_entry_account_id")
                    .table(WorkTimeEntry::Table)
                    .col(WorkTimeEntry::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_work_time_entry_start_time")
                    .table(WorkTimeEntry::Table)
                    .col(WorkTimeEntry::StartTime)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_work_time_entry_is_active")
                    .table(WorkTimeEntry::Table)
                    .col(WorkTimeEntry::IsActive)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(WorkTimeEntry::Table).to_owned())
            .await?;
        
        manager
            .drop_table(Table::drop().table(UserRole::Table).to_owned())
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
enum UserRole {
    Table,
    Id,
    AccountId,
    RoleName,
    HourlyWage,
    Currency,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WorkTimeEntry {
    Table,
    Id,
    AccountId,
    UserRoleId,
    StartTime,
    EndTime,
    Duration,
    Description,
    Project,
    IsActive,
    CreatedAt,
    UpdatedAt,
}