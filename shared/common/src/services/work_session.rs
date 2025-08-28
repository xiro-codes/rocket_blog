use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use models::{work_session, prelude::{WorkSession, WorkRole}, dto::{ClockInFormDTO, WorkSessionWithRoleDTO}};
use crate::services::BaseService;
use std::str::FromStr;
use rust_decimal::Decimal;

pub struct WorkSessionService {
    base: BaseService,
}

impl WorkSessionService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    /// Clock in - start a new work session
    pub async fn clock_in(&self, db: &DbConn, account_id: Uuid, data: ClockInFormDTO) -> Result<work_session::Model, DbErr> {
        log::info!("Account {} attempting to clock in for role {}", account_id, data.work_role_id);
        
        // Check if user is already clocked in
        if let Some(active_session) = self.find_active_session(db, account_id).await? {
            log::warn!("Account {} already has an active session: {}", account_id, active_session.id);
            return Err(DbErr::Custom("Already clocked in. Please clock out first.".to_owned()));
        }

        let work_role_id = Uuid::from_str(&data.work_role_id)
            .map_err(|e| {
                log::error!("Invalid work role ID format: {}", e);
                DbErr::Custom("Invalid work role ID".to_owned())
            })?;

        // Verify the role exists and is active
        let role = WorkRole::find_by_id(work_role_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Work role not found".to_owned()))?;

        if !role.is_active {
            return Err(DbErr::Custom("Work role is not active".to_owned()));
        }

        let now = Utc::now().naive_utc();
        let session = work_session::ActiveModel {
            id: Set(BaseService::generate_id()),
            account_id: Set(account_id),
            work_role_id: Set(work_role_id),
            clock_in_time: Set(now),
            clock_out_time: Set(None),
            duration_minutes: Set(None),
            earnings: Set(None),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = session.insert(db).await?;
        log::info!("Successfully clocked in account {} for session {}", account_id, result.id);
        Ok(result)
    }

    /// Clock out - end the active work session
    pub async fn clock_out(&self, db: &DbConn, account_id: Uuid) -> Result<work_session::Model, DbErr> {
        log::info!("Account {} attempting to clock out", account_id);
        
        let active_session = self.find_active_session(db, account_id).await?
            .ok_or(DbErr::Custom("No active session found".to_owned()))?;

        // Get the role to calculate earnings
        let role = WorkRole::find_by_id(active_session.work_role_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Work role not found".to_owned()))?;

        let clock_out_time = Utc::now().naive_utc();
        let duration = clock_out_time.signed_duration_since(active_session.clock_in_time);
        let duration_minutes = duration.num_minutes() as i32;
        
        // Calculate earnings: (minutes / 60) * hourly_rate
        let hours_worked = Decimal::from(duration_minutes) / Decimal::from(60);
        let earnings = hours_worked * role.hourly_rate;

        let mut session: work_session::ActiveModel = active_session.into();
        session.clock_out_time = Set(Some(clock_out_time));
        session.duration_minutes = Set(Some(duration_minutes));
        session.earnings = Set(Some(earnings));
        session.updated_at = Set(clock_out_time);

        let result = session.update(db).await?;
        log::info!("Successfully clocked out account {} for session {}, worked {} minutes, earned ${}", 
                  account_id, result.id, duration_minutes, earnings);
        Ok(result)
    }

    /// Find active session for an account
    pub async fn find_active_session(&self, db: &DbConn, account_id: Uuid) -> Result<Option<work_session::Model>, DbErr> {
        WorkSession::find()
            .filter(work_session::Column::AccountId.eq(account_id))
            .filter(work_session::Column::ClockOutTime.is_null())
            .one(db)
            .await
    }

    /// Find all sessions for an account with role information
    pub async fn find_sessions_with_role(&self, db: &DbConn, account_id: Uuid) -> Result<Vec<WorkSessionWithRoleDTO>, DbErr> {
        let sessions = WorkSession::find()
            .filter(work_session::Column::AccountId.eq(account_id))
            .find_with_related(WorkRole)
            .order_by_desc(work_session::Column::ClockInTime)
            .all(db)
            .await?;

        let mut result = Vec::new();
        for (session, roles) in sessions {
            if let Some(role) = roles.first() {
                result.push(WorkSessionWithRoleDTO {
                    id: session.id,
                    account_id: session.account_id,
                    work_role_id: session.work_role_id,
                    clock_in_time: session.clock_in_time,
                    clock_out_time: session.clock_out_time,
                    duration_minutes: session.duration_minutes,
                    earnings: session.earnings,
                    role_name: role.name.clone(),
                    hourly_rate: role.hourly_rate,
                });
            }
        }
        
        Ok(result)
    }

    /// Get work summary for an account (total hours and earnings)
    pub async fn get_work_summary(&self, db: &DbConn, account_id: Uuid) -> Result<(i32, Decimal), DbErr> {
        let sessions = WorkSession::find()
            .filter(work_session::Column::AccountId.eq(account_id))
            .filter(work_session::Column::ClockOutTime.is_not_null())
            .all(db)
            .await?;

        let total_minutes: i32 = sessions.iter()
            .filter_map(|s| s.duration_minutes)
            .sum();

        let total_earnings: Decimal = sessions.iter()
            .filter_map(|s| s.earnings)
            .sum();

        Ok((total_minutes, total_earnings))
    }

    /// Get today's work summary for an account
    pub async fn get_today_summary(&self, db: &DbConn, account_id: Uuid) -> Result<(i32, Decimal), DbErr> {
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_end = Utc::now().date_naive().and_hms_opt(23, 59, 59).unwrap();

        let sessions = WorkSession::find()
            .filter(work_session::Column::AccountId.eq(account_id))
            .filter(work_session::Column::ClockInTime.gte(today_start))
            .filter(work_session::Column::ClockInTime.lte(today_end))
            .filter(work_session::Column::ClockOutTime.is_not_null())
            .all(db)
            .await?;

        let total_minutes: i32 = sessions.iter()
            .filter_map(|s| s.duration_minutes)
            .sum();

        let total_earnings: Decimal = sessions.iter()
            .filter_map(|s| s.earnings)
            .sum();

        Ok((total_minutes, total_earnings))
    }
}