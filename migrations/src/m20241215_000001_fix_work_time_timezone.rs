use sea_orm_migration::prelude::*;
use sea_orm::DatabaseBackend;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert work_time_entry timestamp columns to timezone-aware timestamps
        // Note: SQLite doesn't support timestamp with timezone, so we handle it differently
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL: Use timestamp with timezone
                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::StartTime)
                                    .timestamp_with_time_zone()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::EndTime)
                                    .timestamp_with_time_zone()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::CreatedAt)
                                    .timestamp_with_time_zone()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::UpdatedAt)
                                    .timestamp_with_time_zone()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                // Also update background_job table timestamps to be timezone-aware
                manager
                    .alter_table(
                        Table::alter()
                            .table(BackgroundJob::Table)
                            .modify_column(
                                ColumnDef::new(BackgroundJob::CreatedAt)
                                    .timestamp_with_time_zone()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(BackgroundJob::Table)
                            .modify_column(
                                ColumnDef::new(BackgroundJob::UpdatedAt)
                                    .timestamp_with_time_zone()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;
            },
            DatabaseBackend::Sqlite => {
                // SQLite: Keep using timestamp (no timezone support, timezone handled in application)
                // No changes needed for SQLite as it already uses timestamp
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to naive timestamps
        let db_backend = manager.get_database_backend();
        
        match db_backend {
            DatabaseBackend::Postgres => {
                // PostgreSQL: Revert to timestamp without timezone
                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::StartTime)
                                    .timestamp()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::EndTime)
                                    .timestamp()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::CreatedAt)
                                    .timestamp()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(WorkTimeEntry::Table)
                            .modify_column(
                                ColumnDef::new(WorkTimeEntry::UpdatedAt)
                                    .timestamp()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                // Revert background_job table timestamps
                manager
                    .alter_table(
                        Table::alter()
                            .table(BackgroundJob::Table)
                            .modify_column(
                                ColumnDef::new(BackgroundJob::CreatedAt)
                                    .timestamp()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;

                manager
                    .alter_table(
                        Table::alter()
                            .table(BackgroundJob::Table)
                            .modify_column(
                                ColumnDef::new(BackgroundJob::UpdatedAt)
                                    .timestamp()
                                    .not_null()
                            )
                            .to_owned(),
                    )
                    .await?;
            },
            DatabaseBackend::Sqlite => {
                // SQLite: No changes needed to revert as we didn't change anything
            },
            _ => {
                return Err(DbErr::Custom("Unsupported database backend".to_string()));
            }
        }

        Ok(())
    }
}

#[derive(DeriveIden)]
enum WorkTimeEntry {
    Table,
    StartTime,
    EndTime,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum BackgroundJob {
    Table,
    CreatedAt,
    UpdatedAt,
}