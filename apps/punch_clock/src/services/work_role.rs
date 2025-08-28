use sea_orm::*;
use uuid::Uuid;
use models::{work_role, prelude::WorkRole, dto::WorkRoleFormDTO};
use common::{services::BaseService, utils::Utils};

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
        // Validate input data
        if let Err(validation_errors) = self.validate_role_data(&data) {
            log::error!("Work role validation failed: {:?}", validation_errors);
            return Err(DbErr::Custom(validation_errors.join(", ")));
        }

        let hourly_rate = Utils::parse_decimal(&data.hourly_rate, "hourly rate")
            .map_err(|e| DbErr::Custom(e))?;

        let now = BaseService::now();
        let id = BaseService::generate_id();
        
        let role = work_role::ActiveModel {
            id: Set(id),
            name: Set(data.name.clone()),
            hourly_rate: Set(hourly_rate),
            is_active: Set(data.is_active),
            created_at: Set(now),
            updated_at: Set(now),
        };

        let result = role.insert(db).await?;
        BaseService::log_entity_creation("WorkRole", id, &format!("{} at ${}/hr", data.name, data.hourly_rate));
        Ok(result)
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
        // Validate input data
        if let Err(validation_errors) = self.validate_role_data(&data) {
            log::error!("Work role validation failed: {:?}", validation_errors);
            return Err(DbErr::Custom(validation_errors.join(", ")));
        }

        let role = BaseService::ensure_exists::<WorkRole>(db, id, "WorkRole").await?;

        let hourly_rate = Utils::parse_decimal(&data.hourly_rate, "hourly rate")
            .map_err(|e| DbErr::Custom(e))?;

        let mut role: work_role::ActiveModel = role.into();
        role.name = Set(data.name.clone());
        role.hourly_rate = Set(hourly_rate);
        role.is_active = Set(data.is_active);
        role.updated_at = Set(BaseService::now());

        let result = role.update(db).await?;
        BaseService::log_entity_update("WorkRole", id, &format!("{} at ${}/hr", data.name, data.hourly_rate));
        Ok(result)
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
        BaseService::log_entity_deletion("WorkRole", id);
        Ok(())
    }

    /// Validate role data using shared utilities
    fn validate_role_data(&self, data: &WorkRoleFormDTO) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate name
        if let Err(error) = Utils::validate_required_string(&data.name, "Role name") {
            errors.push(error);
        }

        // Validate hourly rate
        if let Ok(rate) = Utils::parse_decimal(&data.hourly_rate, "hourly rate") {
            if let Err(error) = Utils::validate_positive_decimal(rate, "Hourly rate") {
                errors.push(error);
            }
        } else {
            errors.push("Invalid hourly rate format".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}