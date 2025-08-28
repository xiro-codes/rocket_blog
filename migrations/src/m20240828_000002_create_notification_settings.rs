use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(NotificationSettings::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NotificationSettings::Id)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::AccountId)
                            .uuid()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::TimeBasedEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::TimeThresholdMinutes)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::EarningsBasedEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::EarningsThreshold)
                            .decimal_len(10, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::Currency)
                            .string()
                            .string_len(3)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::DailyGoalEnabled)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::DailyHoursGoal)
                            .decimal_len(4, 2)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(NotificationSettings::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_notification_settings_account")
                            .from(NotificationSettings::Table, NotificationSettings::AccountId)
                            .to(Alias::new("account"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .index(
                        Index::create()
                            .name("idx_notification_settings_account_id")
                            .col(NotificationSettings::AccountId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(NotificationSettings::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum NotificationSettings {
    Table,
    Id,
    AccountId,
    TimeBasedEnabled,
    TimeThresholdMinutes,
    EarningsBasedEnabled,
    EarningsThreshold,
    Currency,
    DailyGoalEnabled,
    DailyHoursGoal,
    CreatedAt,
    UpdatedAt,
}