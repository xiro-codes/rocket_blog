#[cfg(test)]
mod tests {
    use crate::pool::{Db, SeaOrmPool};
    use crate::tests::utils::create_test_figment;
    use rocket::figment::{providers::Serialized, Figment};
    use sea_orm_rocket::Config;

    #[test]
    fn test_db_debug_impl() {
        // Test that Db implements Debug trait
        let pool = SeaOrmPool {
            conn: sea_orm::MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection(),
        };
        let db = Db(pool);
        
        let debug_str = format!("{:?}", db);
        assert!(debug_str.contains("Db"));
    }

    #[test]
    fn test_sea_orm_pool_debug_impl() {
        // Test that SeaOrmPool implements Debug trait
        let pool = SeaOrmPool {
            conn: sea_orm::MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection(),
        };
        
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("SeaOrmPool"));
    }

    #[test]
    fn test_sea_orm_pool_clone() {
        // Test that SeaOrmPool implements Clone trait
        let pool = SeaOrmPool {
            conn: sea_orm::MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection(),
        };
        
        let cloned_pool = pool.clone();
        // Both pools should have the same type
        assert_eq!(
            std::mem::discriminant(&pool.conn),
            std::mem::discriminant(&cloned_pool.conn)
        );
    }

    #[test]
    fn test_sea_orm_pool_borrow() {
        let pool = SeaOrmPool {
            conn: sea_orm::MockDatabase::new(sea_orm::DatabaseBackend::Postgres).into_connection(),
        };
        
        let borrowed_conn = pool.borrow();
        // Should return reference to the same connection
        assert!(std::ptr::eq(&pool.conn, borrowed_conn));
    }

    #[tokio::test]
    async fn test_sea_orm_pool_init_with_mock_config() {
        // Create a test configuration
        let figment = rocket::Config::default()
            .figment()
            .merge(Serialized::default("databases.sea_orm.url", "postgres://test:test@localhost/test"))
            .merge(Serialized::default("databases.sea_orm.max_connections", 10))
            .merge(Serialized::default("databases.sea_orm.min_connections", 1))
            .merge(Serialized::default("databases.sea_orm.connect_timeout", 30))
            .merge(Serialized::default("databases.sea_orm.sqlx_logging", false));

        let config = figment.extract::<Config>().unwrap();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, Some(1));
        assert_eq!(config.connect_timeout, 30);
        assert!(!config.sqlx_logging);
    }
}