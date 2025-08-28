use migrations::MigratorTrait;
use rocket::{fairing, Build, Rocket};
use sea_orm_rocket::{Database, rocket::figment::Figment, Config};
use sea_orm::ConnectOptions;
use std::time::Duration;
use std::sync::{Arc, Mutex};

/// Database connection type
#[derive(Database, Debug)]
#[database("sea_orm")]
pub struct Db(SeaOrmPool);

/// Database health status
#[derive(Debug, Clone)]
pub struct DatabaseHealth {
    pub is_available: Arc<Mutex<bool>>,
    pub last_error: Arc<Mutex<Option<String>>>,
}

impl DatabaseHealth {
    pub fn new() -> Self {
        Self {
            is_available: Arc::new(Mutex::new(false)),
            last_error: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_available(&self, available: bool) {
        if let Ok(mut status) = self.is_available.lock() {
            *status = available;
        }
    }

    pub fn set_error(&self, error: Option<String>) {
        if let Ok(mut err) = self.last_error.lock() {
            *err = error;
        }
    }

    pub fn is_healthy(&self) -> bool {
        self.is_available.lock().map(|status| *status).unwrap_or(false)
    }

    pub fn get_last_error(&self) -> Option<String> {
        self.last_error.lock().ok().and_then(|err| err.clone())
    }
}

#[derive(Debug)]
pub struct SeaOrmPool {
    pub conn: sea_orm::DatabaseConnection,
}

#[rocket::async_trait]
impl sea_orm_rocket::Pool for SeaOrmPool {
    type Error = sea_orm::DbErr;
    type Connection = sea_orm::DatabaseConnection;

    async fn init(figment: &Figment) -> Result<Self, Self::Error> {
        let config = figment.extract::<Config>().unwrap();
        let mut options: ConnectOptions = config.url.into();
        options
            .max_connections(config.max_connections as u32)
            .min_connections(config.min_connections.unwrap_or_default())
            .connect_timeout(Duration::from_secs(config.connect_timeout))
            .sqlx_logging(config.sqlx_logging);
        if let Some(idle_timeout) = config.idle_timeout {
            options.idle_timeout(Duration::from_secs(idle_timeout));
        }
        let conn = sea_orm::Database::connect(options).await?;
        Ok(SeaOrmPool { conn })
    }

    fn borrow(&self) -> &Self::Connection {
        &self.conn
    }
}

/// Run database migrations
pub async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    log::info!("Starting database migrations...");
    
    // Check if database is available first
    let db_health = match rocket.state::<DatabaseHealth>() {
        Some(health) => health,
        None => {
            log::warn!("Database health checker not found, skipping migrations");
            return Ok(rocket);
        }
    };

    if !db_health.is_healthy() {
        log::warn!("Database not available, skipping migrations");
        return Ok(rocket);
    }

    match Db::fetch(&rocket) {
        Some(db) => {
            let conn = &db.conn;
            match migrations::Migrator::up(conn, None).await {
                Ok(_) => {
                    log::info!("Database migrations completed successfully");
                }
                Err(e) => {
                    log::error!("Database migration failed: {}", e);
                    db_health.set_error(Some(format!("Migration failed: {}", e)));
                }
            }
        }
        None => {
            log::warn!("Database connection not available for migrations");
        }
    }
    
    Ok(rocket)
}

/// Initialize database with graceful failure handling
pub fn init_with_health() -> (impl fairing::Fairing, DatabaseHealth) {
    let db_health = DatabaseHealth::new();
    let health_clone = db_health.clone();
    
    let fairing = fairing::AdHoc::on_liftoff("Database Health Check", move |rocket| {
        let health = health_clone.clone();
        Box::pin(async move {
            // Test database connection
            match Db::fetch(rocket) {
                Some(_) => {
                    log::info!("Database connection established successfully");
                    health.set_available(true);
                }
                None => {
                    log::warn!("Database connection failed - running in degraded mode");
                    health.set_available(false);
                    health.set_error(Some("Database connection not available".to_string()));
                }
            }
        })
    });
    
    (fairing, db_health)
}