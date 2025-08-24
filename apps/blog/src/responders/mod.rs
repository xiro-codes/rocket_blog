use rocket::response::{Flash, Redirect, Responder};
use rocket::Request;
use rocket_dyn_templates::{context, Template};

/// Custom error responder that provides consistent error handling
#[derive(Debug)]
pub enum ApiResponse {
    /// Success redirect with flash message
    SuccessRedirect(String, String),
    /// Error redirect with flash message  
    ErrorRedirect(String, String),
    /// Template response for error pages
    ErrorTemplate(String, String),
}

impl<'r> Responder<'r, 'static> for ApiResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        match self {
            ApiResponse::SuccessRedirect(url, message) => {
                Flash::success(Redirect::to(url), message).respond_to(_request)
            }
            ApiResponse::ErrorRedirect(url, message) => {
                Flash::new(Redirect::to(url), "danger", message).respond_to(_request)
            }
            ApiResponse::ErrorTemplate(title, message) => {
                Template::render(
                    "error",
                    context! {
                        title: title,
                        message: message
                    }
                ).respond_to(_request)
            }
        }
    }
}

impl ApiResponse {
    /// Create a success redirect response
    pub fn success_redirect<T: Into<String>, U: Into<String>>(url: T, message: U) -> Self {
        Self::SuccessRedirect(url.into(), message.into())
    }

    /// Create an error redirect response
    pub fn error_redirect<T: Into<String>, U: Into<String>>(url: T, message: U) -> Self {
        Self::ErrorRedirect(url.into(), message.into())
    }

    /// Create an error template response
    pub fn error_template<T: Into<String>, U: Into<String>>(title: T, message: U) -> Self {
        Self::ErrorTemplate(title.into(), message.into())
    }
}