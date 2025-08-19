use rocket::http::{CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use uuid::Uuid;

pub mod admin;

/// Request guard for authenticated users
pub struct AuthenticatedUser {
    pub token: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => return Outcome::Error((Status::BadRequest, "Cookie jar not available")),
        };
        
        if let Some(token_cookie) = jar.get_private("token") {
            if let Ok(token) = Uuid::parse_str(token_cookie.value()) {
                return Outcome::Success(AuthenticatedUser { token });
            }
        }
        
        Outcome::Error((Status::Unauthorized, "Authentication required"))
    }
}

/// Request guard for optional authentication
pub struct OptionalUser {
    pub token: Option<Uuid>,
}

#[rocket::async_trait] 
impl<'r> FromRequest<'r> for OptionalUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => return Outcome::Success(OptionalUser { token: None }),
        };
        
        let token = jar.get_private("token")
            .and_then(|cookie| Uuid::parse_str(cookie.value()).ok());
            
        Outcome::Success(OptionalUser { token })
    }
}

/// Admin user request guard 
pub struct AdminUser {
    pub token: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // First check if user is authenticated
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => return Outcome::Error((Status::BadRequest, "Cookie jar not available")),
        };
        
        let token = match jar.get_private("token") {
            Some(cookie) => match Uuid::parse_str(cookie.value()) {
                Ok(uuid) => uuid,
                Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid token")),
            },
            None => return Outcome::Error((Status::Unauthorized, "Authentication required")),
        };

        // For now, we'll assume any authenticated user is an admin
        // TODO: In a real application, check database for admin status
        Outcome::Success(AdminUser { token })
    }
}