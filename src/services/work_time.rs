use chrono::Utc;
use models::{
    dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO, WorkTimeEntryWithRoleDTO},
    user_role, work_time_entry,
};
use rust_decimal::Decimal;
use sea_orm::*;
use uuid::Uuid;
use std::str::FromStr;

use crate::services::BaseService;

type DbConn = DatabaseConnection;

pub struct WorkTimeService {
    base: BaseService,
}

impl WorkTimeService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    // User Role Management
    pub async fn create_user_role(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: UserRoleFormDTO,
    ) -> Result<user_role::Model, DbErr> {
        log::info!("Creating user role '{}' for account {}", data.role_name, account_id);
        
        // Parse hourly wage from string
        let hourly_wage = Decimal::from_str(&data.hourly_wage)
            .map_err(|_| DbErr::Custom("Invalid hourly wage format".to_string()))?;
        
        let role_id = BaseService::generate_id();
        let now = Utc::now();
        
        let user_role = user_role::ActiveModel {
            id: Set(role_id),
            account_id: Set(account_id),
            role_name: Set(data.role_name.clone()),
            hourly_wage: Set(hourly_wage),
            currency: Set(data.currency.clone()),
            is_active: Set(true),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        }
        .insert(db)
        .await?;

        log::info!("User role created successfully: {} ({})", data.role_name, role_id);
        Ok(user_role)
    }

    pub async fn get_user_roles(&self, db: &DbConn, account_id: Uuid) -> Result<Vec<user_role::Model>, DbErr> {
        user_role::Entity::find()
            .filter(user_role::Column::AccountId.eq(account_id))
            .filter(user_role::Column::IsActive.eq(true))
            .order_by_asc(user_role::Column::RoleName)
            .all(db)
            .await
    }

    pub async fn update_user_role(
        &self,
        db: &DbConn,
        role_id: Uuid,
        account_id: Uuid,
        data: UserRoleFormDTO,
    ) -> Result<user_role::Model, DbErr> {
        let role = user_role::Entity::find_by_id(role_id)
            .filter(user_role::Column::AccountId.eq(account_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User role not found".to_string()))?;

        // Parse hourly wage from string
        let hourly_wage = Decimal::from_str(&data.hourly_wage)
            .map_err(|_| DbErr::Custom("Invalid hourly wage format".to_string()))?;

        let mut role: user_role::ActiveModel = role.into();
        role.role_name = Set(data.role_name);
        role.hourly_wage = Set(hourly_wage);
        role.currency = Set(data.currency);
        role.updated_at = Set(Utc::now().naive_utc());

        role.update(db).await
    }

    pub async fn delete_user_role(&self, db: &DbConn, role_id: Uuid, account_id: Uuid) -> Result<(), DbErr> {
        // Soft delete by setting is_active to false
        let role = user_role::Entity::find_by_id(role_id)
            .filter(user_role::Column::AccountId.eq(account_id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("User role not found".to_string()))?;

        let mut role: user_role::ActiveModel = role.into();
        role.is_active = Set(false);
        role.updated_at = Set(Utc::now().naive_utc());

        role.update(db).await?;
        Ok(())
    }

    // Work Time Entry Management
    pub async fn start_time_tracking(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: TimeTrackingControlDTO,
    ) -> Result<work_time_entry::Model, DbErr> {
        log::info!("Starting time tracking for account {} with role {}", account_id, data.user_role_id);
        
        // Check if there's already an active entry for this user
        if let Some(_active_entry) = self.get_active_entry(db, account_id).await? {
            return Err(DbErr::Custom("An active time entry already exists. Stop it first.".to_string()));
        }

        // Verify the role belongs to the user
        let _role = user_role::Entity::find_by_id(data.user_role_id)
            .filter(user_role::Column::AccountId.eq(account_id))
            .filter(user_role::Column::IsActive.eq(true))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("User role not found or inactive".to_string()))?;

        let entry_id = BaseService::generate_id();
        let now = Utc::now();
        
        let entry = work_time_entry::ActiveModel {
            id: Set(entry_id),
            account_id: Set(account_id),
            user_role_id: Set(data.user_role_id),
            start_time: Set(now.naive_utc()),
            end_time: Set(None),
            duration: Set(None),
            description: Set(data.description.clone()),
            project: Set(data.project.clone()),
            is_active: Set(true),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        }
        .insert(db)
        .await?;

        log::info!("Time tracking started: {}", entry_id);
        Ok(entry)
    }

    pub async fn stop_time_tracking(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<work_time_entry::Model, DbErr> {
        log::info!("Stopping time tracking for account {}", account_id);
        
        let active_entry = self.get_active_entry(db, account_id).await?
            .ok_or(DbErr::Custom("No active time entry found".to_string()))?;

        let end_time = Utc::now();
        let duration = (end_time - active_entry.start_time.and_utc()).num_minutes() as i32;

        let mut entry: work_time_entry::ActiveModel = active_entry.into();
        entry.end_time = Set(Some(end_time.naive_utc()));
        entry.duration = Set(Some(duration));
        entry.is_active = Set(false);
        entry.updated_at = Set(end_time.naive_utc());

        let stopped_entry = entry.update(db).await?;
        log::info!("Time tracking stopped. Duration: {} minutes", duration);
        Ok(stopped_entry)
    }

    pub async fn get_active_entry(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<Option<work_time_entry::Model>, DbErr> {
        work_time_entry::Entity::find()
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::IsActive.eq(true))
            .one(db)
            .await
    }

    pub async fn create_manual_entry(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: WorkTimeEntryFormDTO,
    ) -> Result<work_time_entry::Model, DbErr> {
        log::info!("Creating manual work time entry for account {}", account_id);
        
        // Verify the role belongs to the user
        let _role = user_role::Entity::find_by_id(data.user_role_id)
            .filter(user_role::Column::AccountId.eq(account_id))
            .filter(user_role::Column::IsActive.eq(true))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("User role not found or inactive".to_string()))?;

        // Parse dates if provided
        let start_time = if let Some(start_str) = data.start_time {
            chrono::DateTime::parse_from_rfc3339(&start_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| DbErr::Custom("Invalid start time format".to_string()))?
        } else {
            Utc::now()
        };

        let end_time = if let Some(end_str) = data.end_time {
            Some(chrono::DateTime::parse_from_rfc3339(&end_str)
                .map(|dt| dt.with_timezone(&Utc))
                .map_err(|_| DbErr::Custom("Invalid end time format".to_string()))?)
        } else {
            None
        };

        let duration = if let Some(end) = end_time {
            Some((end - start_time).num_minutes() as i32)
        } else {
            None
        };

        let entry_id = BaseService::generate_id();
        let now = Utc::now();
        
        let entry = work_time_entry::ActiveModel {
            id: Set(entry_id),
            account_id: Set(account_id),
            user_role_id: Set(data.user_role_id),
            start_time: Set(start_time.naive_utc()),
            end_time: Set(end_time.map(|t| t.naive_utc())),
            duration: Set(duration),
            description: Set(data.description.clone()),
            project: Set(data.project.clone()),
            is_active: Set(false),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        }
        .insert(db)
        .await?;

        log::info!("Manual work time entry created: {}", entry_id);
        Ok(entry)
    }

    pub async fn get_work_entries_with_roles(
        &self,
        db: &DbConn,
        account_id: Uuid,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<WorkTimeEntryWithRoleDTO>, DbErr> {
        let mut query = work_time_entry::Entity::find()
            .find_also_related(user_role::Entity)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .order_by_desc(work_time_entry::Column::StartTime);

        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let entries = query.all(db).await?;

        let mut result = Vec::new();
        for (entry, role) in entries {
            if let Some(role) = role {
                let earnings = if let Some(duration) = entry.duration {
                    let hours = Decimal::from(duration) / Decimal::from(60);
                    Some(hours * role.hourly_wage)
                } else {
                    None
                };

                result.push(WorkTimeEntryWithRoleDTO {
                    id: entry.id,
                    start_time: entry.start_time.and_utc(),
                    end_time: entry.end_time.map(|t| t.and_utc()),
                    duration: entry.duration,
                    description: entry.description,
                    project: entry.project,
                    is_active: entry.is_active,
                    role_name: role.role_name,
                    hourly_wage: role.hourly_wage,
                    currency: role.currency,
                    earnings,
                });
            }
        }

        Ok(result)
    }

    pub async fn get_work_time_summary(
        &self,
        db: &DbConn,
        account_id: Uuid,
        start_date: Option<chrono::DateTime<Utc>>,
        end_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<WorkTimeSummaryDTO, DbErr> {
        let mut query = work_time_entry::Entity::find()
            .find_also_related(user_role::Entity)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::IsActive.eq(false)); // Only completed entries

        if let Some(start) = start_date {
            query = query.filter(work_time_entry::Column::StartTime.gte(start));
        }
        if let Some(end) = end_date {
            query = query.filter(work_time_entry::Column::StartTime.lte(end));
        }

        let entries = query.all(db).await?;

        let mut total_hours = Decimal::from(0);
        let mut total_earnings = Decimal::from(0);
        let mut currency = "USD".to_string();
        let entries_count = entries.len() as i32;

        for (entry, role) in entries {
            if let (Some(role), Some(duration)) = (role, entry.duration) {
                let hours = Decimal::from(duration) / Decimal::from(60);
                total_hours += hours;
                total_earnings += hours * role.hourly_wage;
                currency = role.currency; // Use the last currency found
            }
        }

        Ok(WorkTimeSummaryDTO {
            total_hours,
            total_earnings,
            currency,
            entries_count,
        })
    }

    pub async fn delete_work_entry(&self, db: &DbConn, entry_id: Uuid, account_id: Uuid) -> Result<(), DbErr> {
        work_time_entry::Entity::delete_by_id(entry_id)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .exec(db)
            .await?;
        
        log::info!("Work time entry deleted: {}", entry_id);
        Ok(())
    }
}