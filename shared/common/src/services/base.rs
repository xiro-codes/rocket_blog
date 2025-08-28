use sea_orm::*;
use std::future::Future;
use uuid::Uuid;
use chrono::Utc;

/// Generic trait for CRUD operations on entities
pub trait CrudService<T, CreateDto, UpdateDto>
where
    T: EntityTrait + Send + Sync,
    T::Model: Send + Sync,
{
    /// Create a new entity
    fn create(
        &self,
        db: &DbConn,
        data: CreateDto,
    ) -> impl Future<Output = Result<T::Model, DbErr>> + Send;

    /// Find an entity by its ID
    fn find_by_id(
        &self,
        db: &DbConn,
        id: Uuid,
    ) -> impl Future<Output = Result<Option<T::Model>, DbErr>> + Send;

    /// Update an entity by its ID
    fn update_by_id(
        &self,
        db: &DbConn,
        id: Uuid,
        data: UpdateDto,
    ) -> impl Future<Output = Result<T::Model, DbErr>> + Send;

    /// Delete an entity by its ID
    fn delete_by_id(
        &self,
        db: &DbConn,
        id: Uuid,
    ) -> impl Future<Output = Result<DeleteResult, DbErr>> + Send;
}

/// Base service struct that provides common functionality
pub struct BaseService;

impl BaseService {
    pub fn new() -> Self {
        Self
    }

    /// Generate a new UUID
    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Generate a timestamp for creation
    pub fn now() -> chrono::NaiveDateTime {
        Utc::now().naive_utc()
    }

    /// Handle common database errors
    pub fn handle_not_found<T>(result: Option<T>, entity_name: &str) -> Result<T, DbErr> {
        result.ok_or_else(|| DbErr::RecordNotFound(format!("{} not found", entity_name)))
    }

    /// Log entity creation with standardized format
    pub fn log_entity_creation(entity_type: &str, entity_id: Uuid, details: &str) {
        log::info!("Created {} {}: {}", entity_type, entity_id, details);
    }

    /// Log entity update with standardized format
    pub fn log_entity_update(entity_type: &str, entity_id: Uuid, details: &str) {
        log::info!("Updated {} {}: {}", entity_type, entity_id, details);
    }

    /// Log entity deletion with standardized format
    pub fn log_entity_deletion(entity_type: &str, entity_id: Uuid) {
        log::info!("Deleted {} {}", entity_type, entity_id);
    }

    /// Convert SeaORM error to string for consistent error handling
    pub fn db_error_to_string(err: DbErr) -> String {
        match err {
            DbErr::RecordNotFound(msg) => format!("Record not found: {}", msg),
            DbErr::Custom(msg) => msg,
            _ => err.to_string(),
        }
    }

    /// Validate entity existence
    pub async fn ensure_exists<E>(
        db: &DbConn,
        id: Uuid,
        entity_name: &str,
    ) -> Result<E::Model, DbErr>
    where
        E: EntityTrait + Send + Sync,
        E::Model: Send + Sync,
        <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
    {
        E::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("{} with id {} not found", entity_name, id)))
    }
}