use chrono::Utc;
use models::{
    dto::{UserRoleFormDTO, WorkTimeEntryFormDTO, TimeTrackingControlDTO, WorkTimeSummaryDTO, WorkTimeEntryWithRoleDTO, WorkTimeEntryDisplayDTO, NotificationSettingsFormDTO, PayPeriodSummaryDTO},
    user_role, work_time_entry, notification_settings, pay_period,
};
use sea_orm::*;
use uuid::Uuid;

use crate::{services::{BaseService, PayPeriodService, TimezoneService}, impl_service_with_base};

type DbConn = DatabaseConnection;

pub struct WorkTimeService {
    base: BaseService,
}

impl WorkTimeService {

    // User Role Management
    pub async fn create_user_role(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: UserRoleFormDTO,
    ) -> Result<user_role::Model, DbErr> {
        log::info!("Creating user role '{}' for account {}", data.role_name, account_id);
        
        // Parse hourly wage from string
        let hourly_wage = data.hourly_wage.parse::<f64>()
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
        let hourly_wage = data.hourly_wage.parse::<f64>()
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
            pay_period_id: Set(None), // Will be set when the entry is completed
            start_time: Set(now),
            end_time: Set(None),
            duration: Set(None),
            description: Set(None),
            project: Set(None),
            is_active: Set(true),
            created_at: Set(now),
            updated_at: Set(now),
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
        let duration = (end_time - active_entry.start_time).num_minutes() as i32;

        // Find the appropriate pay period for this entry
        let entry_date = active_entry.start_time.date_naive();
        let pay_period = pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .filter(pay_period::Column::StartDate.lte(entry_date))
            .filter(pay_period::Column::EndDate.gte(entry_date))
            .one(db)
            .await?;

        let mut entry: work_time_entry::ActiveModel = active_entry.into();
        entry.end_time = Set(Some(end_time));
        entry.duration = Set(Some(duration));
        entry.pay_period_id = Set(pay_period.as_ref().map(|p| p.id));
        entry.is_active = Set(false);
        entry.updated_at = Set(end_time);

        let stopped_entry = entry.update(db).await?;
        log::info!("Time tracking stopped. Duration: {} minutes", duration);
        
        if let Some(pay_period) = pay_period {
            log::info!("Assigned entry to pay period: {}", pay_period.period_name);
        }
        
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

        // Find the appropriate pay period for this entry
        let entry_date = start_time.date_naive();
        let pay_period = pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .filter(pay_period::Column::StartDate.lte(entry_date))
            .filter(pay_period::Column::EndDate.gte(entry_date))
            .one(db)
            .await?;
        
        let entry = work_time_entry::ActiveModel {
            id: Set(entry_id),
            account_id: Set(account_id),
            user_role_id: Set(data.user_role_id),
            pay_period_id: Set(pay_period.as_ref().map(|p| p.id)),
            start_time: Set(start_time),
            end_time: Set(end_time),
            duration: Set(duration),
            description: Set(None),
            project: Set(None),
            is_active: Set(false),
            created_at: Set(now),
            updated_at: Set(now),
        }
        .insert(db)
        .await?;

        log::info!("Manual work time entry created: {}", entry_id);
        
        if let Some(pay_period) = pay_period {
            log::info!("Assigned manual entry to pay period: {}", pay_period.period_name);
        }
        
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
                    let hours = duration as f64 / 60.0;
                    Some(hours * role.hourly_wage)
                } else {
                    None
                };

                result.push(WorkTimeEntryWithRoleDTO {
                    id: entry.id,
                    start_time: entry.start_time,
                    end_time: entry.end_time,
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

        let mut total_hours = 0.0;
        let mut total_earnings = 0.0;
        let mut currency = "USD".to_string();
        let entries_count = entries.len() as i32;

        for (entry, role) in entries {
            if let (Some(role), Some(duration)) = (role, entry.duration) {
                let hours = duration as f64 / 60.0;
                total_hours += hours;
                total_earnings += hours * role.hourly_wage;
                currency = role.currency; // Use the last currency found
            }
        }

        // Get current shift earnings (active entry if any)
        let current_shift_earnings = self.get_current_shift_earnings(db, account_id).await.unwrap_or(0.0);
        
        // Get pay period hours (current pay period)
        let pay_period_hours = self.get_current_pay_period_hours(db, account_id).await.unwrap_or(0.0);

        Ok(WorkTimeSummaryDTO {
            total_hours,
            total_earnings,
            currency,
            entries_count,
            current_shift_earnings,
            pay_period_hours,
        })
    }

    /// Get current shift earnings (for active timer)
    pub async fn get_current_shift_earnings(&self, db: &DbConn, account_id: Uuid) -> Result<f64, DbErr> {
        if let Some(active_entry) = self.get_active_entry(db, account_id).await? {
            // Calculate earnings based on elapsed time and hourly rate
            let now = Utc::now();
            let elapsed_minutes = (now - active_entry.start_time).num_minutes() as f64;
            let elapsed_hours = elapsed_minutes / 60.0;
            
            // Get the role for hourly wage
            if let Some(role) = user_role::Entity::find_by_id(active_entry.user_role_id)
                .one(db)
                .await? {
                return Ok(elapsed_hours * role.hourly_wage);
            }
        }
        Ok(0.0)
    }

    /// Get total hours worked in current pay period
    pub async fn get_current_pay_period_hours(&self, db: &DbConn, account_id: Uuid) -> Result<f64, DbErr> {
        use chrono::{Datelike, Duration, Weekday};
        
        let now = Utc::now().date_naive();
        
        // For now, assume pay periods start on Monday and last 2 weeks
        // This could be made configurable using the settings
        let days_since_monday = match now.weekday() {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        };
        
        let current_monday = now - Duration::days(days_since_monday);
        
        // Calculate current pay period start (could be current Monday or previous Monday depending on bi-weekly cycle)
        // For simplicity, let's use the current Monday as the start for now
        let pay_period_start = current_monday.and_hms_opt(0, 0, 0).unwrap();
        let pay_period_start_utc = pay_period_start.and_utc();
        
        // Get all completed entries in this pay period
        let entries = work_time_entry::Entity::find()
            .find_also_related(user_role::Entity)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::IsActive.eq(false))
            .filter(work_time_entry::Column::StartTime.gte(pay_period_start_utc))
            .all(db)
            .await?;

        let mut total_hours = 0.0;
        for (entry, _) in entries {
            if let Some(duration) = entry.duration {
                total_hours += duration as f64 / 60.0;
            }
        }

        Ok(total_hours)
    }

    pub async fn delete_work_entry(&self, db: &DbConn, entry_id: Uuid, account_id: Uuid) -> Result<(), DbErr> {
        work_time_entry::Entity::delete_by_id(entry_id)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .exec(db)
            .await?;
        
        log::info!("Work time entry deleted: {}", entry_id);
        Ok(())
    }

    pub async fn get_work_entry_by_id(&self, db: &DbConn, entry_id: Uuid, account_id: Uuid) -> Result<Option<work_time_entry::Model>, DbErr> {
        work_time_entry::Entity::find_by_id(entry_id)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .one(db)
            .await
    }

    pub async fn update_work_entry(
        &self,
        db: &DbConn,
        entry_id: Uuid,
        account_id: Uuid,
        data: WorkTimeEntryFormDTO,
    ) -> Result<work_time_entry::Model, DbErr> {
        log::info!("Updating work time entry {} for account {}", entry_id, account_id);
        
        // Get the existing entry
        let entry = self.get_work_entry_by_id(db, entry_id, account_id).await?
            .ok_or(DbErr::RecordNotFound("Work time entry not found".to_string()))?;

        // Verify the new role belongs to the user
        let _role = user_role::Entity::find_by_id(data.user_role_id)
            .filter(user_role::Column::AccountId.eq(account_id))
            .filter(user_role::Column::IsActive.eq(true))
            .one(db)
            .await?
            .ok_or(DbErr::Custom("User role not found or inactive".to_string()))?;

        // Parse the datetime strings
        let start_time = if let Some(start_str) = data.start_time {
            chrono::NaiveDateTime::parse_from_str(&start_str, "%Y-%m-%dT%H:%M")
                .map_err(|_| DbErr::Custom("Invalid start time format".to_string()))?
                .and_utc()
        } else {
            entry.start_time
        };

        let end_time = if let Some(end_str) = data.end_time {
            Some(chrono::NaiveDateTime::parse_from_str(&end_str, "%Y-%m-%dT%H:%M")
                .map_err(|_| DbErr::Custom("Invalid end time format".to_string()))?
                .and_utc())
        } else {
            entry.end_time
        };

        // Calculate duration if both times are provided
        let duration = if let Some(end) = end_time {
            Some((end - start_time).num_minutes() as i32)
        } else {
            None
        };

        // Update the entry
        let mut active_entry: work_time_entry::ActiveModel = entry.into();
        active_entry.user_role_id = Set(data.user_role_id);
        active_entry.start_time = Set(start_time);
        active_entry.end_time = Set(end_time);
        active_entry.duration = Set(duration);
        active_entry.is_active = Set(end_time.is_none()); // Active if no end time
        active_entry.updated_at = Set(Utc::now());

        let updated_entry = active_entry.update(db).await?;
        log::info!("Work time entry updated: {}", entry_id);
        Ok(updated_entry)
    }

    // Notification Settings Management
    pub async fn get_notification_settings(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<Option<notification_settings::Model>, DbErr> {
        notification_settings::Entity::find()
            .filter(notification_settings::Column::AccountId.eq(account_id))
            .one(db)
            .await
    }

    pub async fn create_or_update_notification_settings(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: NotificationSettingsFormDTO,
    ) -> Result<notification_settings::Model, DbErr> {
        log::info!("Creating/updating notification settings for account {}", account_id);

        // Parse numeric values from strings
        let time_threshold_minutes = if let Some(val) = data.time_threshold_minutes {
            if val.is_empty() { None } else {
                Some(val.parse::<i32>().map_err(|_| DbErr::Custom("Invalid time threshold format".to_string()))?)
            }
        } else {
            None
        };

        let earnings_threshold = if let Some(val) = data.earnings_threshold {
            if val.is_empty() { None } else {
                Some(val.parse::<f64>().map_err(|_| DbErr::Custom("Invalid earnings threshold format".to_string()))?)
            }
        } else {
            None
        };

        let daily_hours_goal = if let Some(val) = data.daily_hours_goal {
            if val.is_empty() { None } else {
                Some(val.parse::<f64>().map_err(|_| DbErr::Custom("Invalid daily hours goal format".to_string()))?)
            }
        } else {
            None
        };

        // Check if settings already exist
        let existing = self.get_notification_settings(db, account_id).await?;
        
        if let Some(existing_settings) = existing {
            // Update existing settings
            let mut active_model: notification_settings::ActiveModel = existing_settings.into();
            active_model.time_based_enabled = Set(data.time_based_enabled.unwrap_or(false));
            active_model.time_threshold_minutes = Set(time_threshold_minutes);
            active_model.earnings_based_enabled = Set(data.earnings_based_enabled.unwrap_or(false));
            active_model.earnings_threshold = Set(earnings_threshold);
            active_model.currency = Set(data.currency);
            active_model.daily_goal_enabled = Set(data.daily_goal_enabled.unwrap_or(false));
            active_model.daily_hours_goal = Set(daily_hours_goal);
            active_model.updated_at = Set(Utc::now());
            
            active_model.update(db).await
        } else {
            // Create new settings
            let settings_id = BaseService::generate_id();
            let now = Utc::now();
            
            let notification_settings = notification_settings::ActiveModel {
                id: Set(settings_id),
                account_id: Set(account_id),
                time_based_enabled: Set(data.time_based_enabled.unwrap_or(false)),
                time_threshold_minutes: Set(time_threshold_minutes),
                earnings_based_enabled: Set(data.earnings_based_enabled.unwrap_or(false)),
                earnings_threshold: Set(earnings_threshold),
                currency: Set(data.currency),
                daily_goal_enabled: Set(data.daily_goal_enabled.unwrap_or(false)),
                daily_hours_goal: Set(daily_hours_goal),
                created_at: Set(now),
                updated_at: Set(now),
            };
            
            notification_settings.insert(db).await
        }
    }

    pub async fn check_notification_triggers(
        &self,
        db: &DbConn,
        account_id: Uuid,
        current_session_duration_minutes: Option<i32>,
        current_session_earnings: Option<f64>,
    ) -> Result<Vec<String>, DbErr> {
        let mut notifications = Vec::new();
        
        // Get notification settings
        if let Some(settings) = self.get_notification_settings(db, account_id).await? {
            // Check time-based notifications
            if settings.time_based_enabled {
                if let (Some(threshold), Some(duration)) = (settings.time_threshold_minutes, current_session_duration_minutes) {
                    if duration >= threshold {
                        notifications.push(format!("You've been working for {} minutes! Time for a break?", duration));
                    }
                }
            }

            // Check earnings-based notifications
            if settings.earnings_based_enabled {
                if let (Some(threshold), Some(earnings)) = (settings.earnings_threshold, current_session_earnings) {
                    if earnings >= threshold {
                        let currency = settings.currency.unwrap_or_else(|| "USD".to_string());
                        notifications.push(format!("Great job! You've earned {} {} in this session!", earnings, currency));
                    }
                }
            }

            // Check daily goal notifications
            if settings.daily_goal_enabled {
                if let Some(daily_goal) = settings.daily_hours_goal {
                    // Get today's total hours
                    let today = Utc::now().date_naive();
                    let tomorrow = today.succ_opt().unwrap_or(today);
                    
                    if let Ok(summary) = self.get_work_time_summary(
                        db, 
                        account_id, 
                        Some(today.and_hms_opt(0, 0, 0).unwrap().and_utc()), 
                        Some(tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc())
                    ).await {
                        if summary.total_hours >= daily_goal {
                            notifications.push(format!("🎉 Daily goal achieved! You've worked {} hours today!", summary.total_hours));
                        }
                    }
                }
            }
        }
        
        Ok(notifications)
    }

    /// Get work time summary for a specific pay period
    pub async fn get_work_time_summary_by_pay_period(
        &self,
        db: &DbConn,
        account_id: Uuid,
        pay_period_id: Option<Uuid>,
    ) -> Result<PayPeriodSummaryDTO, DbErr> {
        let mut query = work_time_entry::Entity::find()
            .find_also_related(user_role::Entity)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::IsActive.eq(false)); // Only completed entries

        if let Some(period_id) = pay_period_id {
            query = query.filter(work_time_entry::Column::PayPeriodId.eq(period_id));
        } else {
            // Get entries without assigned pay period
            query = query.filter(work_time_entry::Column::PayPeriodId.is_null());
        }

        let entries = query.all(db).await?;

        let mut total_hours = 0.0;
        let mut total_earnings = 0.0;
        let mut currency = "USD".to_string();
        let entries_count = entries.len() as i32;

        for (entry, role) in entries {
            if let (Some(role), Some(duration)) = (role, entry.duration) {
                let hours = duration as f64 / 60.0;
                total_hours += hours;
                total_earnings += hours * role.hourly_wage;
                currency = role.currency; // Use the last currency found
            }
        }

        // Get pay period info if provided
        let (pay_period_name, period_start_date, period_end_date) = if let Some(period_id) = pay_period_id {
            let pay_period = pay_period::Entity::find_by_id(period_id)
                .filter(pay_period::Column::AccountId.eq(account_id))
                .one(db)
                .await?;
            
            if let Some(period) = pay_period {
                (Some(period.period_name), Some(period.start_date), Some(period.end_date))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        Ok(PayPeriodSummaryDTO {
            pay_period_id,
            pay_period_name,
            period_start_date,
            period_end_date,
            total_hours,
            total_earnings,
            currency,
            entries_count,
        })
    }

    /// Convert WorkTimeEntryWithRoleDTO to WorkTimeEntryDisplayDTO with timezone formatting
    pub fn format_entries_for_display(
        entries: Vec<WorkTimeEntryWithRoleDTO>,
        user_timezone: &str,
    ) -> Vec<WorkTimeEntryDisplayDTO> {
        entries.into_iter().map(|entry| {
            let start_time_display = TimezoneService::format_with_timezone(entry.start_time, user_timezone)
                .unwrap_or_else(|_| entry.start_time.format("%Y-%m-%d %H:%M:%S UTC").to_string());
            
            let end_time_display = entry.end_time.map(|end_time| {
                TimezoneService::format_with_timezone(end_time, user_timezone)
                    .unwrap_or_else(|_| end_time.format("%Y-%m-%d %H:%M:%S UTC").to_string())
            });
            
            WorkTimeEntryDisplayDTO {
                id: entry.id,
                start_time: entry.start_time,
                end_time: entry.end_time,
                start_time_display,
                end_time_display,
                duration: entry.duration,
                description: entry.description,
                project: entry.project,
                is_active: entry.is_active,
                role_name: entry.role_name,
                hourly_wage: entry.hourly_wage,
                currency: entry.currency,
                earnings: entry.earnings,
            }
        }).collect()
    }

    /// Get work entries with timezone-aware display formatting
    pub async fn get_work_entries_for_display(
        &self,
        db: &DbConn,
        account_id: Uuid,
        user_timezone: &str,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<Vec<WorkTimeEntryDisplayDTO>, DbErr> {
        let entries = self.get_work_entries_with_roles(db, account_id, limit, offset).await?;
        Ok(Self::format_entries_for_display(entries, user_timezone))
    }
}

impl_service_with_base!(WorkTimeService);