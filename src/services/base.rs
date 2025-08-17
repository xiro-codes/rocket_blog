use sea_orm::*;
use std::future::Future;
use uuid::Uuid;

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
#[derive(Clone)]
pub struct BaseService;

impl BaseService {
    pub fn new() -> Self {
        Self
    }

    /// Generate a new UUID
    pub fn generate_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Handle common database errors
    pub fn handle_not_found<T>(result: Option<T>, entity_name: &str) -> Result<T, DbErr> {
        result.ok_or_else(|| DbErr::RecordNotFound(format!("{} not found", entity_name)))
    }
}
