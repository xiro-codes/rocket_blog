use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use models::{work_role, prelude::WorkRole, dto::WorkRoleFormDTO};
use common::services::BaseService;
use rust_decimal::Decimal;
use std::str::FromStr;

pub struct WorkRoleService {
    base: BaseService,
}

impl WorkRoleService {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    /// Create a new work role
    pub async fn create(&self, db: &DbConn, data: WorkRoleFormDTO) -> Result<work_role::Model, DbErr> {
        log::info!("Creating new work role: {}", data.name);
        
        let hourly_rate = Decimal::from_str(&data.hourly_rate)
            .map_err(|e| {
                log::error!("Invalid hourly rate format: {}", e);
                DbErr::Custom("Invalid hourly rate format".to_owned())
            })?;

        let now = Utc::now().naive_utc();
        let role = work_role::ActiveModel {
            id: Set(BaseService::generate_id()),
            name: Set(data.name),
            hourly_rate: Set(hourly_rate),
            is_active: Set(data.is_active),
            created_at: Set(now),
            updated_at: Set(now),
        };

        role.insert(db).await
    }

    /// Find all work roles
    pub async fn find_all(&self, db: &DbConn) -> Result<Vec<work_role::Model>, DbErr> {
        WorkRole::find()
            .order_by_asc(work_role::Column::Name)
            .all(db)
            .await
    }

    /// Find active work roles
    pub async fn find_active(&self, db: &DbConn) -> Result<Vec<work_role::Model>, DbErr> {
        WorkRole::find()
            .filter(work_role::Column::IsActive.eq(true))
            .order_by_asc(work_role::Column::Name)
            .all(db)
            .await
    }

    /// Find work role by ID
    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<work_role::Model>, DbErr> {
        WorkRole::find_by_id(id).one(db).await
    }

    /// Update work role
    pub async fn update(&self, db: &DbConn, id: Uuid, data: WorkRoleFormDTO) -> Result<work_role::Model, DbErr> {
        let role = self.find_by_id(db, id).await?
            .ok_or(DbErr::RecordNotFound(format!("Work role with id: {}", id)))?;

        let hourly_rate = Decimal::from_str(&data.hourly_rate)
            .map_err(|e| {
                log::error!("Invalid hourly rate format: {}", e);
                DbErr::Custom("Invalid hourly rate format".to_owned())
            })?;

        let mut role: work_role::ActiveModel = role.into();
        role.name = Set(data.name);
        role.hourly_rate = Set(hourly_rate);
        role.is_active = Set(data.is_active);
        role.updated_at = Set(Utc::now().naive_utc());

        role.update(db).await
    }

    /// Delete work role (only if no sessions exist)
    pub async fn delete(&self, db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        // Check if any work sessions exist for this role
        let session_count = models::prelude::WorkSession::find()
            .filter(models::work_session::Column::WorkRoleId.eq(id))
            .count(db)
            .await?;

        if session_count > 0 {
            return Err(DbErr::Custom("Cannot delete work role with existing sessions".to_owned()));
        }

        WorkRole::delete_by_id(id).exec(db).await?;
        Ok(())
    }
}