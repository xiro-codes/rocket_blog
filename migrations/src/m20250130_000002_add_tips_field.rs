use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add tips column to work_time_entry table
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTimeEntry::Table)
                    .add_column(ColumnDef::new(WorkTimeEntry::Tips).double().null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Remove tips column from work_time_entry table
        manager
            .alter_table(
                Table::alter()
                    .table(WorkTimeEntry::Table)
                    .drop_column(WorkTimeEntry::Tips)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum WorkTimeEntry {
    Table,
    Tips,
}