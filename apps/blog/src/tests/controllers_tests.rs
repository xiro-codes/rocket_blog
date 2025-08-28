#[cfg(test)]
mod tests {
    use crate::controllers::{AuthController, BlogController, CommentController, IndexController, FeedController, ControllerBase};
    use uuid::Uuid;

    mod controller_base_tests {
        use super::*;

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

    mod feed_controller_tests {
        use super::*;

        #[test]
        fn test_feed_controller_new() {
            let path = "/feed".to_string();
            let controller = FeedController::new(path.clone());
            
            // Should create successfully
            assert_eq!(std::mem::size_of_val(&controller), std::mem::size_of::<FeedController>());
        }
    }
}