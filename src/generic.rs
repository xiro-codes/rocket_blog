//! Generic traits and utilities for controllers and services to reduce code duplication

use rocket::{
    fairing::{Fairing, Info, Kind, Result as FairingResult},
    http::{CookieJar, Status},
    Build, Rocket, Route, State,
};
use sea_orm::{DbConn, DbErr};
use uuid::Uuid;

use crate::services::AuthService;

/// Generic trait for CRUD operations that most services implement
pub trait CrudService<Model, CreateDto, UpdateDto, Id = Uuid> {
    /// Create a new entity
    async fn create(&self, db: &DbConn, data: CreateDto) -> Result<Model, DbErr>;
    
    /// Find entity by ID
    async fn find_by_id(&self, db: &DbConn, id: Id) -> Result<Option<Model>, DbErr>;
    
    /// Update entity by ID
    async fn update_by_id(&self, db: &DbConn, id: Id, data: UpdateDto) -> Result<Model, DbErr>;
    
    /// Delete entity by ID
    async fn delete_by_id(&self, db: &DbConn, id: Id) -> Result<(), DbErr>;
}

/// Generic trait for controllers to implement common functionality
pub trait GenericController: Sized {
    type Service;
    
    /// Get the mount path for this controller
    fn path(&self) -> &str;
    
    /// Get the controller name for fairing info
    fn name() -> &'static str;
    
    /// Get routes for this controller
    fn routes() -> Vec<Route>;
    
    /// Create new instance of the service
    fn create_service() -> Self::Service;
}

/// Generic controller implementation that provides common fairing behavior
pub struct BaseController<T> {
    pub path: String,
    pub _phantom: std::marker::PhantomData<T>,
}

impl<T> BaseController<T> {
    pub fn new(path: String) -> Self {
        Self {
            path,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[rocket::async_trait]
impl<T> Fairing for BaseController<T>
where
    T: GenericController + Send + Sync + 'static,
    T::Service: Send + Sync + 'static,
{
    fn info(&self) -> Info {
        Info {
            name: T::name(),
            kind: Kind::Ignite,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> FairingResult {
        Ok(rocket
            .manage(T::create_service())
            .mount(self.path.clone(), T::routes()))
    }
}

/// Authentication utilities to reduce duplication
pub struct AuthUtils;

impl AuthUtils {
    /// Check if user is authenticated and return token
    pub fn get_token(jar: &CookieJar<'_>) -> Option<Uuid> {
        jar.get_private("token")
            .and_then(|c| Uuid::parse_str(c.value()).ok())
    }
    
    /// Check if user is authenticated and is admin - takes database directly
    pub async fn check_admin_auth_with_db(
        db: &sea_orm::DatabaseConnection,
        auth_service: &State<AuthService>,
        jar: &CookieJar<'_>,
    ) -> Result<models::account::Model, Status> {
        let token = Self::get_token(jar).ok_or(Status::Unauthorized)?;
        
        match auth_service.check_token(db, token).await {
            Some(account) if account.admin => Ok(account),
            Some(_) => Err(Status::Forbidden),
            None => Err(Status::Unauthorized),
        }
    }
    
    /// Check if user is authenticated (doesn't need to be admin) - takes database directly
    pub async fn check_auth_with_db(
        db: &sea_orm::DatabaseConnection,
        auth_service: &State<AuthService>, 
        jar: &CookieJar<'_>,
    ) -> Result<models::account::Model, Status> {
        let token = Self::get_token(jar).ok_or(Status::Unauthorized)?;
        
        auth_service
            .check_token(db, token)
            .await
            .ok_or(Status::Unauthorized)
    }
}

/// Generic pagination utilities
pub struct PaginationUtils;

impl PaginationUtils {
    pub const DEFAULT_PAGE_SIZE: u64 = 39;
    
    /// Validate and normalize page parameters
    pub fn normalize_pagination(page: Option<u64>, page_size: Option<u64>) -> Result<(u64, u64), DbErr> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(Self::DEFAULT_PAGE_SIZE);
        
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
        
        Ok((page, page_size))
    }
}