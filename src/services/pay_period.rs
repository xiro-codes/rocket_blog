use chrono::{Utc, NaiveDate};
use models::{
    dto::{PayPeriodFormDTO, PayPeriodWithSummaryDTO, PayPeriodSummaryDTO},
    pay_period, work_time_entry, user_role,
};
use sea_orm::*;
use uuid::Uuid;

use crate::services::BaseService;

type DbConn = DatabaseConnection;

pub struct PayPeriodService {
    base: BaseService,
}

impl PayPeriodService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    /// Create a new pay period
    pub async fn create_pay_period(
        &self,
        db: &DbConn,
        account_id: Uuid,
        data: PayPeriodFormDTO,
    ) -> Result<pay_period::Model, DbErr> {
        log::info!("Creating pay period '{}' for account {}", data.period_name, account_id);
        
        // Parse dates from strings
        let start_date = NaiveDate::parse_from_str(&data.start_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("Invalid start date format. Use YYYY-MM-DD".to_string()))?;
        
        let end_date = NaiveDate::parse_from_str(&data.end_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("Invalid end date format. Use YYYY-MM-DD".to_string()))?;

        // Validate that start_date is before end_date
        if start_date >= end_date {
            return Err(DbErr::Custom("Start date must be before end date".to_string()));
        }

        // Check for overlapping pay periods
        let overlapping = pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.lte(start_date))
                            .add(pay_period::Column::EndDate.gte(start_date))
                    )
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.lte(end_date))
                            .add(pay_period::Column::EndDate.gte(end_date))
                    )
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.gte(start_date))
                            .add(pay_period::Column::EndDate.lte(end_date))
                    )
            )
            .count(db)
            .await?;

        if overlapping > 0 {
            return Err(DbErr::Custom("Pay period overlaps with existing active pay period".to_string()));
        }
        
        let period_id = BaseService::generate_id();
        let now = Utc::now();
        
        let pay_period = pay_period::ActiveModel {
            id: Set(period_id),
            account_id: Set(account_id),
            period_name: Set(data.period_name.clone()),
            start_date: Set(start_date),
            end_date: Set(end_date),
            is_active: Set(true),
            created_at: Set(now.naive_utc()),
            updated_at: Set(now.naive_utc()),
        }
        .insert(db)
        .await?;

        log::info!("Pay period created successfully: {} ({})", data.period_name, period_id);
        Ok(pay_period)
    }

    /// Get all pay periods for a user
    pub async fn get_user_pay_periods(
        &self, 
        db: &DbConn, 
        account_id: Uuid
    ) -> Result<Vec<pay_period::Model>, DbErr> {
        pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .order_by_desc(pay_period::Column::StartDate)
            .all(db)
            .await
    }

    /// Get pay periods with summary information
    pub async fn get_pay_periods_with_summary(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<Vec<PayPeriodWithSummaryDTO>, DbErr> {
        let pay_periods = self.get_user_pay_periods(db, account_id).await?;
        let today = Utc::now().date_naive();
        
        let mut result = Vec::new();
        
        for period in pay_periods {
            // Calculate summary for this pay period
            let entries = work_time_entry::Entity::find()
                .find_also_related(user_role::Entity)
                .filter(work_time_entry::Column::AccountId.eq(account_id))
                .filter(work_time_entry::Column::PayPeriodId.eq(period.id))
                .filter(work_time_entry::Column::IsActive.eq(false)) // Only completed entries
                .all(db)
                .await?;

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

            let is_current = today >= period.start_date && today <= period.end_date;

            result.push(PayPeriodWithSummaryDTO {
                id: period.id,
                period_name: period.period_name,
                start_date: period.start_date,
                end_date: period.end_date,
                is_active: period.is_active,
                created_at: period.created_at.and_utc(),
                total_hours,
                total_earnings,
                currency,
                entries_count,
                is_current,
            });
        }

        Ok(result)
    }

    /// Get current pay period (the one that includes today's date)
    pub async fn get_current_pay_period(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<Option<pay_period::Model>, DbErr> {
        let today = Utc::now().date_naive();
        
        pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .filter(pay_period::Column::StartDate.lte(today))
            .filter(pay_period::Column::EndDate.gte(today))
            .one(db)
            .await
    }

    /// Get pay period by ID (ensuring it belongs to the user)
    pub async fn get_pay_period_by_id(
        &self,
        db: &DbConn,
        period_id: Uuid,
        account_id: Uuid,
    ) -> Result<Option<pay_period::Model>, DbErr> {
        pay_period::Entity::find_by_id(period_id)
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .one(db)
            .await
    }

    /// Update a pay period
    pub async fn update_pay_period(
        &self,
        db: &DbConn,
        period_id: Uuid,
        account_id: Uuid,
        data: PayPeriodFormDTO,
    ) -> Result<pay_period::Model, DbErr> {
        let period = self.get_pay_period_by_id(db, period_id, account_id).await?
            .ok_or(DbErr::RecordNotFound("Pay period not found".to_string()))?;

        // Parse dates from strings
        let start_date = NaiveDate::parse_from_str(&data.start_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("Invalid start date format. Use YYYY-MM-DD".to_string()))?;
        
        let end_date = NaiveDate::parse_from_str(&data.end_date, "%Y-%m-%d")
            .map_err(|_| DbErr::Custom("Invalid end date format. Use YYYY-MM-DD".to_string()))?;

        // Validate that start_date is before end_date
        if start_date >= end_date {
            return Err(DbErr::Custom("Start date must be before end date".to_string()));
        }

        // Check for overlapping pay periods (excluding current one)
        let overlapping = pay_period::Entity::find()
            .filter(pay_period::Column::AccountId.eq(account_id))
            .filter(pay_period::Column::IsActive.eq(true))
            .filter(pay_period::Column::Id.ne(period_id))
            .filter(
                Condition::any()
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.lte(start_date))
                            .add(pay_period::Column::EndDate.gte(start_date))
                    )
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.lte(end_date))
                            .add(pay_period::Column::EndDate.gte(end_date))
                    )
                    .add(
                        Condition::all()
                            .add(pay_period::Column::StartDate.gte(start_date))
                            .add(pay_period::Column::EndDate.lte(end_date))
                    )
            )
            .count(db)
            .await?;

        if overlapping > 0 {
            return Err(DbErr::Custom("Pay period overlaps with existing active pay period".to_string()));
        }

        let mut active_model: pay_period::ActiveModel = period.into();
        active_model.period_name = Set(data.period_name);
        active_model.start_date = Set(start_date);
        active_model.end_date = Set(end_date);
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(db).await
    }

    /// Soft delete a pay period
    pub async fn delete_pay_period(
        &self,
        db: &DbConn,
        period_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), DbErr> {
        let period = self.get_pay_period_by_id(db, period_id, account_id).await?
            .ok_or(DbErr::RecordNotFound("Pay period not found".to_string()))?;

        let mut active_model: pay_period::ActiveModel = period.into();
        active_model.is_active = Set(false);
        active_model.updated_at = Set(Utc::now().naive_utc());

        active_model.update(db).await?;
        
        log::info!("Pay period soft deleted: {}", period_id);
        Ok(())
    }

    /// Auto-assign work time entries to appropriate pay periods
    pub async fn auto_assign_entries_to_pay_periods(
        &self,
        db: &DbConn,
        account_id: Uuid,
    ) -> Result<i32, DbErr> {
        log::info!("Auto-assigning work time entries to pay periods for account {}", account_id);
        
        // Get all unassigned entries
        let unassigned_entries = work_time_entry::Entity::find()
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::PayPeriodId.is_null())
            .filter(work_time_entry::Column::IsActive.eq(false)) // Only completed entries
            .all(db)
            .await?;

        let mut assigned_count = 0;

        for entry in unassigned_entries {
            let entry_date = entry.start_time.date();
            
            // Find the pay period that contains this entry's date
            if let Some(pay_period) = pay_period::Entity::find()
                .filter(pay_period::Column::AccountId.eq(account_id))
                .filter(pay_period::Column::IsActive.eq(true))
                .filter(pay_period::Column::StartDate.lte(entry_date))
                .filter(pay_period::Column::EndDate.gte(entry_date))
                .one(db)
                .await? {
                
                // Update the entry with the pay period
                let mut active_entry: work_time_entry::ActiveModel = entry.into();
                active_entry.pay_period_id = Set(Some(pay_period.id));
                active_entry.updated_at = Set(Utc::now().naive_utc());
                
                active_entry.update(db).await?;
                assigned_count += 1;
            }
        }

        log::info!("Auto-assigned {} work time entries to pay periods", assigned_count);
        Ok(assigned_count)
    }

    /// Get pay period summary for a specific pay period
    pub async fn get_pay_period_summary(
        &self,
        db: &DbConn,
        period_id: Uuid,
        account_id: Uuid,
    ) -> Result<PayPeriodSummaryDTO, DbErr> {
        let pay_period = self.get_pay_period_by_id(db, period_id, account_id).await?
            .ok_or(DbErr::RecordNotFound("Pay period not found".to_string()))?;

        // Get all entries for this pay period
        let entries = work_time_entry::Entity::find()
            .find_also_related(user_role::Entity)
            .filter(work_time_entry::Column::AccountId.eq(account_id))
            .filter(work_time_entry::Column::PayPeriodId.eq(period_id))
            .filter(work_time_entry::Column::IsActive.eq(false)) // Only completed entries
            .all(db)
            .await?;

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

        Ok(PayPeriodSummaryDTO {
            pay_period_id: Some(pay_period.id),
            pay_period_name: Some(pay_period.period_name),
            period_start_date: Some(pay_period.start_date),
            period_end_date: Some(pay_period.end_date),
            total_hours,
            total_earnings,
            currency,
            entries_count,
        })
    }
}