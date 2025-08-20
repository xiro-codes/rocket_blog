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

        // For now, we'll assume any authenticated user is an admin
        // TODO: In a real application, check database for admin status
        log::debug!("AdminUser guard: granting admin access (TODO: implement proper admin check)");
        Outcome::Success(AdminUser { token })
    }
}