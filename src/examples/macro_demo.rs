//! Demonstration of the new macro and trait system
//! 
//! This module showcases how the new macros and traits can be used to reduce
//! boilerplate and create more maintainable code.

use crate::{
    create_service_registry, create_controller_registry, impl_service_with_base,
    impl_controller, auth_required, admin_required,
    services::{AuthService, BlogService, CommentService, TagService, SettingsService, BaseService},
    controllers::{AuthController, BlogController, CommentController, IndexController, ControllerBase}
};

// Example of using the service registry macro
create_service_registry!(
    DemoServiceRegistry, 
    [
        AuthService,
        BlogService, 
        CommentService,
        TagService,
        SettingsService,
    ]
);

// Example of using the controller registry macro
create_controller_registry!(
    DemoControllerRegistry,
    [
        (AuthController, "/auth"),
        (BlogController, "/blog"),
        (CommentController, "/comment"), 
        (IndexController, "/"),
    ]
);

// Example service using the new patterns
pub struct DemoService {
    base: BaseService,
}

impl_service_with_base!(DemoService);

impl DemoService {
    pub async fn demo_endpoint(
        &self,
        jar: &rocket::http::CookieJar<'_>,
    ) -> Result<String, rocket::http::Status> {
        // Using the auth_required macro
        let _token = auth_required!(jar);
        
        Ok("Demo endpoint accessed successfully".to_string())
    }
    
    pub async fn admin_endpoint(
        &self,
        conn: sea_orm_rocket::Connection<'_, crate::pool::Db>,
        coordinator: &rocket::State<crate::services::CoordinatorService>,
        jar: &rocket::http::CookieJar<'_>,
    ) -> Result<String, rocket::http::Status> {
        // Using the admin_required macro
        admin_required!(conn, coordinator, jar);
        
        Ok("Admin endpoint accessed successfully".to_string())
    }
}

// Example controller using the new patterns
pub struct DemoController {
    base: ControllerBase,
}

impl DemoController {
    pub fn new(path: String) -> Self {
        Self {
            base: ControllerBase::new(path),
        }
    }
}

impl_controller!(DemoController, "Demo Controller", rocket::routes![]);

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_service_registry_creation() {
        // Test that the macro creates the registry properly
        let _registry = DemoServiceRegistry;
    }
    
    #[test]
    fn test_controller_registry_creation() {
        // Test that the macro creates the registry properly
        let _registry = DemoControllerRegistry;
    }
    
    #[test]
    fn test_demo_service_creation() {
        let service = DemoService::new();
        assert_eq!(std::mem::size_of_val(&service), std::mem::size_of::<DemoService>());
    }
    
    #[test]
    fn test_demo_controller_creation() {
        let controller = DemoController::new("/demo".to_string());
        assert_eq!(controller.base.path(), "/demo");
    }
}