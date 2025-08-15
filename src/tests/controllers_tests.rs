#[cfg(test)]
mod tests {
    use crate::controllers::{AuthController, BlogController, CommentController, IndexController, ControllerBase};
    use crate::tests::utils::create_test_rocket;
    use rocket::http::{Status, Cookie, SameSite};
    use rocket::testing::MockRequest;
    use rocket::local::blocking::{Client, LocalRequest};
    use uuid::Uuid;

    mod controller_base_tests {
        use super::*;
        use rocket::http::CookieJar;

        #[test]
        fn test_controller_base_new() {
            let path = "/test".to_string();
            let controller = ControllerBase::new(path.clone());
            
            assert_eq!(controller.path(), &path);
        }

        #[test]
        fn test_controller_base_path() {
            let path = "/api/v1/test".to_string();
            let controller = ControllerBase::new(path.clone());
            
            assert_eq!(controller.path(), "/api/v1/test");
        }

        #[test]
        fn test_check_auth_no_token() {
            let rocket = create_test_rocket();
            let client = Client::tracked(rocket).expect("valid rocket instance");
            let request = client.get("/");
            
            // Mock cookie jar without token
            let jar = request.cookies();
            let result = ControllerBase::check_auth(&jar);
            
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        }

        #[test]
        fn test_require_auth_no_token() {
            let rocket = create_test_rocket();
            let client = Client::tracked(rocket).expect("valid rocket instance");
            let request = client.get("/");
            
            let jar = request.cookies();
            let result = ControllerBase::require_auth(&jar);
            
            assert!(result.is_err());
            assert_eq!(result.unwrap_err(), Status::Unauthorized);
        }
    }

    mod auth_controller_tests {
        use super::*;

        #[test]
        fn test_auth_controller_new() {
            let path = "/auth".to_string();
            let controller = AuthController::new(path.clone());
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&controller), std::mem::size_of::<AuthController>());
        }
    }

    mod blog_controller_tests {
        use super::*;

        #[test]
        fn test_blog_controller_new() {
            let path = "/blog".to_string();
            let controller = BlogController::new(path.clone());
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&controller), std::mem::size_of::<BlogController>());
        }
    }

    mod comment_controller_tests {
        use super::*;

        #[test] 
        fn test_comment_controller_new() {
            let path = "/comment".to_string();
            let controller = CommentController::new(path.clone());
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&controller), std::mem::size_of::<CommentController>());
        }
    }

    mod index_controller_tests {
        use super::*;

        #[test]
        fn test_index_controller_new() {
            let path = "/".to_string();
            let controller = IndexController::new(path.clone());
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&controller), std::mem::size_of::<IndexController>());
        }
    }

    // Integration tests for controller endpoints
    mod integration_tests {
        use super::*;
        use crate::services::TagService;
        use crate::config::AppConfig;
        use rocket::State;

        #[tokio::test]
        async fn test_rocket_build_with_controllers() {
            // Test that rocket can be built with all controllers attached
            let rocket = rocket::build()
                .manage(TagService::new())
                .manage(AppConfig::default())
                .attach(IndexController::new("/".to_owned()))
                .attach(AuthController::new("/auth".to_owned()))
                .attach(BlogController::new("/blog".to_owned()))
                .attach(CommentController::new("/comment".to_owned()));

            // Should build without panicking
            assert!(rocket.state::<TagService>().is_some());
            assert!(rocket.state::<AppConfig>().is_some());
        }
    }
}