use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Convert work_time_entry timestamp columns to timezone-aware timestamps
        
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

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Revert back to naive timestamps
        
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