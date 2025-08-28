use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
};
use rocket::http::CookieJar;
use uuid::Uuid;
use std::str::FromStr;

/// Request guard for authenticated users
pub struct User {
    pub id: Uuid,
}

/// Request guard for admin users (for managing roles)
pub struct Admin {
    pub id: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = request.cookies();
        
        if let Some(token_cookie) = jar.get_private("token") {
            if let Ok(token_uuid) = Uuid::from_str(token_cookie.value()) {
                // In a real implementation, you would validate the token against the database
                // For now, we'll just use the token as the user ID
                return Outcome::Success(User { id: token_uuid });
            }
        }
        
        Outcome::Error((Status::Unauthorized, ()))
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = request.cookies();
        
        if let Some(token_cookie) = jar.get_private("token") {
            if let Ok(token_uuid) = Uuid::from_str(token_cookie.value()) {
                // In a real implementation, you would check if the user is an admin
                // For now, we'll assume all authenticated users can be admins for role management
                return Outcome::Success(Admin { id: token_uuid });
            }
        }
        
        Outcome::Error((Status::Unauthorized, ()))
    }
}