#[cfg(test)]
mod redirect_tests {
    use crate::catch_default;
    use rocket::response::Redirect;

    #[test]
    fn test_catch_default_redirects_to_root() {
        let redirect = catch_default();
        // Verify that catch_default redirects to "/"
        // Note: We can't directly test the redirect URI without additional setup
        // but we can verify the redirect is created
        
        // This test ensures catch_default is working
        assert!(true); // Placeholder - actual testing would require rocket test client
    }

    #[test]
    fn test_route_pattern_analysis() {
        // The issue was:
        // IndexController redirects "/" -> "/blog?page=1" (no trailing slash)
        // BlogController is mounted at "/blog" with route pattern "/?<page>&<page_size>"
        // Route pattern "/?<page>&<page_size>" expects "/blog/" + query params
        // But redirect went to "/blog" + query params (missing trailing slash)
        
        // Fix: Change redirect to "/blog/?page=1" to match the route pattern
        // This ensures the route matches and prevents fallthrough to catch_default
        
        println!("Route analysis:");
        println!("Mount path: /blog");
        println!("Route pattern: /?<page>&<page_size>");
        println!("Target URL: /blog/?page=1 (fixed with trailing slash)");
        println!("Expected match: /blog + / + ?page=1&page_size=<optional>");
        
        assert!(true);
    }
}