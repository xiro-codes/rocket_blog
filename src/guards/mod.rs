use rocket::http::{CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use uuid::Uuid;
use crate::services::AuthService;

pub mod admin;

/// Request guard for authenticated users
pub struct AuthenticatedUser {
    pub token: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        log::debug!("AuthenticatedUser guard: checking authentication");
        
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => {
                log::debug!("AuthenticatedUser guard: cookie jar not available");
                return Outcome::Error((Status::BadRequest, "Cookie jar not available"));
            }
        };
        
        if let Some(token_cookie) = jar.get_private("token") {
            log::debug!("AuthenticatedUser guard: found token cookie");
            if let Ok(token) = Uuid::parse_str(token_cookie.value()) {
                log::debug!("AuthenticatedUser guard: valid token format: {}", token);
                return Outcome::Success(AuthenticatedUser { token });
            } else {
                log::debug!("AuthenticatedUser guard: invalid token format");
            }
        } else {
            log::debug!("AuthenticatedUser guard: no token cookie found");
        }
        
        log::debug!("AuthenticatedUser guard: authentication failed");
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
        log::debug!("OptionalUser guard: checking optional authentication");
        
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => {
                log::debug!("OptionalUser guard: no cookie jar, returning None");
                return Outcome::Success(OptionalUser { token: None });
            }
        };
        
        let token = jar.get_private("token")
            .and_then(|cookie| {
                match Uuid::parse_str(cookie.value()) {
                    Ok(uuid) => {
                        log::debug!("OptionalUser guard: found valid token: {}", uuid);
                        Some(uuid)
                    }
                    Err(_) => {
                        log::debug!("OptionalUser guard: found invalid token format");
                        None
                    }
                }
            });
            
        if token.is_none() {
            log::debug!("OptionalUser guard: no valid token found");
        }
        
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
        log::debug!("AdminUser guard: checking admin authentication");
        
        // First check if user is authenticated
        let jar = match req.guard::<&CookieJar<'_>>().await.succeeded() {
            Some(jar) => jar,
            None => {
                log::debug!("AdminUser guard: cookie jar not available");
                return Outcome::Error((Status::BadRequest, "Cookie jar not available"));
            }
        };
        
        let token = match jar.get_private("token") {
            Some(cookie) => match Uuid::parse_str(cookie.value()) {
                Ok(uuid) => {
                    log::debug!("AdminUser guard: found valid token: {}", uuid);
                    uuid
                }
                Err(_) => {
                    log::debug!("AdminUser guard: invalid token format");
                    return Outcome::Error((Status::Unauthorized, "Invalid token"));
                }
            },
            None => {
                log::debug!("AdminUser guard: no token found");
                return Outcome::Error((Status::Unauthorized, "Authentication required"));
            }
        };

        // Get the auth service and check if user is admin
        let auth_service = match req.guard::<&State<AuthService>>().await.succeeded() {
            Some(service) => service,
            None => {
                log::error!("AdminUser guard: AuthService not available");
                return Outcome::Error((Status::InternalServerError, "Service unavailable"));
            }
        };

        // Get database connection
        let conn = match req.guard::<sea_orm_rocket::Connection<'_, crate::pool::Db>>().await.succeeded() {
            Some(conn) => conn,
            None => {
                log::error!("AdminUser guard: Database connection not available");
                return Outcome::Error((Status::InternalServerError, "Database unavailable"));
            }
        };

        let db = conn.into_inner();
        
        // Check if token belongs to an admin
        if let Some(account) = auth_service.check_token(db, token).await {
            if account.admin {
                log::debug!("AdminUser guard: admin access granted for user: {}", account.username);
                return Outcome::Success(AdminUser { token });
            } else {
                log::debug!("AdminUser guard: access denied - user {} is not an admin", account.username);
                return Outcome::Error((Status::Forbidden, "Admin access required"));
            }
        } else {
            log::debug!("AdminUser guard: invalid token or user not found");
            return Outcome::Error((Status::Unauthorized, "Invalid token"));
        }
    }
}