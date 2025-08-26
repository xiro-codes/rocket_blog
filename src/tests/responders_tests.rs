#[cfg(test)]
mod tests {
    use crate::responders::ApiResponse;
    use rocket::http::Status;

    #[test]
    fn test_api_response_success_redirect_creation() {
        let response = ApiResponse::success_redirect("/dashboard", "Operation successful");
        
        match response {
            ApiResponse::SuccessRedirect(url, message) => {
                assert_eq!(url, "/dashboard");
                assert_eq!(message, "Operation successful");
            }
            _ => panic!("Expected SuccessRedirect variant"),
        }
    }

    #[test]
    fn test_api_response_error_redirect_creation() {
        let response = ApiResponse::error_redirect("/login", "Authentication failed");
        
        match response {
            ApiResponse::ErrorRedirect(url, message) => {
                assert_eq!(url, "/login");
                assert_eq!(message, "Authentication failed");
            }
            _ => panic!("Expected ErrorRedirect variant"),
        }
    }

    #[test]
    fn test_api_response_error_template_creation() {
        let response = ApiResponse::error_template("Not Found", "The requested resource was not found");
        
        match response {
            ApiResponse::ErrorTemplate(title, message) => {
                assert_eq!(title, "Not Found");
                assert_eq!(message, "The requested resource was not found");
            }
            _ => panic!("Expected ErrorTemplate variant"),
        }
    }

    #[test]
    fn test_api_response_with_string_conversions() {
        // Test that the methods accept Into<String> parameters
        let url = String::from("/test");
        let message = String::from("Test message");
        
        let response1 = ApiResponse::success_redirect(&url, &message);
        let response2 = ApiResponse::error_redirect(url.clone(), message.clone());
        let response3 = ApiResponse::error_template(&url, &message);
        
        // Verify the conversions work correctly
        match response1 {
            ApiResponse::SuccessRedirect(u, m) => {
                assert_eq!(u, "/test");
                assert_eq!(m, "Test message");
            }
            _ => panic!("Expected SuccessRedirect"),
        }
        
        match response2 {
            ApiResponse::ErrorRedirect(u, m) => {
                assert_eq!(u, "/test");
                assert_eq!(m, "Test message");
            }
            _ => panic!("Expected ErrorRedirect"),
        }
        
        match response3 {
            ApiResponse::ErrorTemplate(t, m) => {
                assert_eq!(t, "/test");
                assert_eq!(m, "Test message");
            }
            _ => panic!("Expected ErrorTemplate"),
        }
    }

    #[test]
    fn test_api_response_empty_strings() {
        let response = ApiResponse::success_redirect("", "");
        
        match response {
            ApiResponse::SuccessRedirect(url, message) => {
                assert_eq!(url, "");
                assert_eq!(message, "");
            }
            _ => panic!("Expected SuccessRedirect variant"),
        }
    }

    #[test]
    fn test_api_response_long_strings() {
        let long_url = "/very/long/path/to/some/resource/that/has/many/segments/and/is/quite/lengthy";
        let long_message = "This is a very long error message that might be displayed to the user and contains detailed information about what went wrong during the operation";
        
        let response = ApiResponse::error_redirect(long_url, long_message);
        
        match response {
            ApiResponse::ErrorRedirect(url, message) => {
                assert_eq!(url, long_url);
                assert_eq!(message, long_message);
            }
            _ => panic!("Expected ErrorRedirect variant"),
        }
    }

    #[test]
    fn test_api_response_special_characters() {
        let url_with_query = "/search?q=test&page=1";
        let message_with_quotes = "Error: \"Invalid input\" provided";
        
        let response = ApiResponse::error_template("Error", message_with_quotes);
        
        match response {
            ApiResponse::ErrorTemplate(title, message) => {
                assert_eq!(title, "Error");
                assert_eq!(message, message_with_quotes);
            }
            _ => panic!("Expected ErrorTemplate variant"),
        }
    }

    #[test]
    fn test_api_response_unicode_strings() {
        let unicode_url = "/тест/пути";
        let unicode_message = "Ошибка: неверный ввод 🚫";
        
        let response = ApiResponse::success_redirect(unicode_url, unicode_message);
        
        match response {
            ApiResponse::SuccessRedirect(url, message) => {
                assert_eq!(url, unicode_url);
                assert_eq!(message, unicode_message);
            }
            _ => panic!("Expected SuccessRedirect variant"),
        }
    }

    #[test]
    fn test_api_response_debug_format() {
        let response = ApiResponse::success_redirect("/test", "test message");
        let debug_string = format!("{:?}", response);
        
        // Should be able to debug format the response
        assert!(debug_string.contains("SuccessRedirect"));
        assert!(debug_string.contains("/test"));
        assert!(debug_string.contains("test message"));
    }

    #[test]
    fn test_api_response_all_variants() {
        // Test that all three variants can be created and matched
        let responses = vec![
            ApiResponse::success_redirect("/success", "Success!"),
            ApiResponse::error_redirect("/error", "Error!"),
            ApiResponse::error_template("Title", "Error!"),
        ];
        
        assert_eq!(responses.len(), 3);
        
        for response in responses {
            match response {
                ApiResponse::SuccessRedirect(_, _) => { /* OK */ },
                ApiResponse::ErrorRedirect(_, _) => { /* OK */ },
                ApiResponse::ErrorTemplate(_, _) => { /* OK */ },
            }
        }
    }
}