use sea_orm_migration::prelude::*;
use sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create pay_periods table for pay period management
        manager
            .create_table(
                Table::create()
                    .table(PayPeriod::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PayPeriod::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(PayPeriod::AccountId).uuid().not_null())
                    .col(ColumnDef::new(PayPeriod::PeriodName).string().not_null())
                    .col(ColumnDef::new(PayPeriod::StartDate).date().not_null())
                    .col(ColumnDef::new(PayPeriod::EndDate).date().not_null())
                    .col(ColumnDef::new(PayPeriod::IsActive).boolean().not_null().default(true))
                    .col(ColumnDef::new(PayPeriod::CreatedAt).timestamp().not_null())
                    .col(ColumnDef::new(PayPeriod::UpdatedAt).timestamp().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("FK_pay_period_account")
                            .from(PayPeriod::Table, PayPeriod::AccountId)
                            .to(Account::Table, Account::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Add pay_period_id to work_time_entry table
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTimeEntry::Table)
                    .add_column(ColumnDef::new(WorkTimeEntry::PayPeriodId).uuid())
                    .to_owned(),
            )
            .await?;

        // Handle foreign key constraints differently based on database backend
        let db_backend = manager.get_database_backend();
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL supports adding foreign keys to existing tables
                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .add_foreign_key(
                                TableForeignKey::new()
                                    .name("FK_work_time_entry_pay_period")
                                    .from_tbl(WorkTimeEntry::Table)
                                    .from_col(WorkTimeEntry::PayPeriodId)
                                    .to_tbl(PayPeriod::Table)
                                    .to_col(PayPeriod::Id)
                                    .on_delete(ForeignKeyAction::SetNull)
                                    .on_update(ForeignKeyAction::Cascade),
                            )
                            .to_owned(),
                    )
                    .await?;
            },
            DatabaseBackend::Sqlite => {
                // SQLite doesn't support adding foreign keys to existing tables
                // Foreign key constraint will be enforced at application level
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        // Create indexes for better performance
        manager
            .create_index(
                Index::create()
                    .name("idx_pay_period_account_id")
                    .table(PayPeriod::Table)
                    .col(PayPeriod::AccountId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_pay_period_dates")
                    .table(PayPeriod::Table)
                    .col(PayPeriod::StartDate)
                    .col(PayPeriod::EndDate)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_work_time_entry_pay_period_id")
                    .table(WorkTimeEntry::Table)
                    .col(WorkTimeEntry::PayPeriodId)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Handle foreign key removal based on database backend
        let db_backend = manager.get_database_backend();
        match db_backend {
            DatabaseBackend::Postgres => {
                // Remove foreign key constraint first (PostgreSQL)
                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .drop_foreign_key(Alias::new("FK_work_time_entry_pay_period"))
                            .to_owned(),
                    )
                    .await?;
            },
            DatabaseBackend::Sqlite => {
                // SQLite: No foreign key to remove as it wasn't added
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        // Remove pay_period_id column from work_time_entry
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTimeEntry::Table)
                    .drop_column(WorkTimeEntry::PayPeriodId)
                    .to_owned(),
            )
            .await?;

        // Drop pay_periods table
        manager
            .drop_table(Table::drop().table(PayPeriod::Table).to_owned())
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
enum PayPeriod {
    Table,
    Id,
    AccountId,
    PeriodName,
    StartDate,
    EndDate,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum WorkTimeEntry {
    Table,
    PayPeriodId,
}