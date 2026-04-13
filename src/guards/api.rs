use rocket::http::Status;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::State;
use uuid::Uuid;
use crate::services::AuthService;
use sea_orm_rocket;

/// Request guard for authenticated API users using Authorization header.
pub struct ApiUser {
    pub token: Uuid,
    pub account_id: Uuid,
    pub username: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiUser {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        log::debug!("ApiUser guard: checking authentication");

        let token = match req.headers().get_one("Authorization") {
            Some(auth) if auth.starts_with("Bearer ") => {
                match Uuid::parse_str(&auth[7..]) {
                    Ok(uuid) => uuid,
                    Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid token format")),
                }
            }
            _ => return Outcome::Error((Status::Unauthorized, "Missing or invalid Authorization header")),
        };

        // Get the auth service and verify token
        let auth_service = match req.guard::<&State<AuthService>>().await.succeeded() {
            Some(service) => service,
            None => {
                log::error!("ApiUser guard: AuthService not available");
                return Outcome::Error((Status::InternalServerError, "Service unavailable"));
            }
        };

        // Get database connection
        let conn = match req.guard::<sea_orm_rocket::Connection<'_, crate::pool::Db>>().await.succeeded() {
            Some(conn) => conn,
            None => {
                log::error!("ApiUser guard: Database connection not available");
                return Outcome::Error((Status::InternalServerError, "Database unavailable"));
            }
        };

        let db = conn.into_inner();

        // Check if token is valid and get account info
        match auth_service.check_token(db, token).await {
            Some(account) => {
                log::debug!("ApiUser guard: authenticated user: {}", account.username);
                Outcome::Success(ApiUser {
                    token,
                    account_id: account.id,
                    username: account.username,
                })
            }
            None => {
                log::debug!("ApiUser guard: invalid token");
                Outcome::Error((Status::Unauthorized, "Invalid token"))
            }
        }
    }
}

/// Request guard for optional API authentication
pub struct OptionalApiUser {
    pub token: Option<Uuid>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for OptionalApiUser {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        log::debug!("OptionalApiUser guard: checking optional authentication");

        let token = match req.headers().get_one("Authorization") {
            Some(auth) if auth.starts_with("Bearer ") => {
                match Uuid::parse_str(&auth[7..]) {
                    Ok(uuid) => Some(uuid),
                    Err(_) => None,
                }
            }
            _ => None,
        };

        Outcome::Success(OptionalApiUser { token })
    }
}

/// API Admin user request guard
pub struct ApiAdmin {
    pub token: Uuid,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiAdmin {
    type Error = &'static str;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        log::debug!("ApiAdmin guard: checking admin authentication");

        let token = match req.headers().get_one("Authorization") {
            Some(auth) if auth.starts_with("Bearer ") => {
                match Uuid::parse_str(&auth[7..]) {
                    Ok(uuid) => uuid,
                    Err(_) => return Outcome::Error((Status::Unauthorized, "Invalid token format")),
                }
            }
            _ => return Outcome::Error((Status::Unauthorized, "Missing or invalid Authorization header")),
        };

        // Get the auth service and check if user is admin
        let auth_service = match req.guard::<&State<AuthService>>().await.succeeded() {
            Some(service) => service,
            None => {
                log::error!("ApiAdmin guard: AuthService not available");
                return Outcome::Error((Status::InternalServerError, "Service unavailable"));
            }
        };

        // Get database connection
        let conn = match req.guard::<sea_orm_rocket::Connection<'_, crate::pool::Db>>().await.succeeded() {
            Some(conn) => conn,
            None => {
                log::error!("ApiAdmin guard: Database connection not available");
                return Outcome::Error((Status::InternalServerError, "Database unavailable"));
            }
        };

        let db = conn.into_inner();

        // Check if token belongs to an admin
        if let Some(account) = auth_service.check_token(db, token).await {
            if account.admin {
                log::debug!("ApiAdmin guard: admin access granted for user: {}", account.username);
                return Outcome::Success(ApiAdmin { token });
            } else {
                log::debug!("ApiAdmin guard: access denied - user {} is not an admin", account.username);
                return Outcome::Error((Status::Forbidden, "Admin access required"));
            }
        } else {
            log::debug!("ApiAdmin guard: invalid token or user not found");
            return Outcome::Error((Status::Unauthorized, "Invalid token"));
        }
    }
}
