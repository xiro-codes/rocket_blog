use rocket::{
    http::Status,
    request::{FromRequest, Outcome, Request},
    State,
};
use rocket::http::CookieJar;
use uuid::Uuid;
use std::str::FromStr;
use sea_orm_rocket::Connection;
use common::{database::Db, auth::AuthService};

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
                // Get database connection and auth service
                if let Some(db_conn) = request.guard::<Connection<Db>>().await.succeeded() {
                    if let Some(auth_service) = request.guard::<&State<AuthService>>().await.succeeded() {
                        let db = db_conn.into_inner();
                        if let Some(account) = auth_service.check_token(db, token_uuid).await {
                            return Outcome::Success(User { id: account.id });
                        }
                    }
                }
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
                // Get database connection and auth service
                if let Some(db_conn) = request.guard::<Connection<Db>>().await.succeeded() {
                    if let Some(auth_service) = request.guard::<&State<AuthService>>().await.succeeded() {
                        let db = db_conn.into_inner();
                        if let Some(account) = auth_service.check_token(db, token_uuid).await {
                            if account.admin {
                                return Outcome::Success(Admin { id: account.id });
                            }
                        }
                    }
                }
            }
        }
        
        Outcome::Error((Status::Unauthorized, ()))
    }
}