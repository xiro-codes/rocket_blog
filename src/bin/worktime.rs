//! Work Time Tracker Application Binary
//! 
//! This binary contains the Progressive Web App for work time tracking including:
//! - Time tracking with role-based wages
//! - Configurable notifications
//! - Pay period management
//! - PWA capabilities with offline support
//! - Independent authentication and operation

use app::{
    features::Features,
    controllers,
    services::{AuthService, WorkTimeService, PayPeriodService, SettingsService},
    create_base_rocket
};
use rocket::{fs::FileServer, response::Redirect, Build, Rocket, Request, Response, catchers, catch, launch, get};
use rocket_dyn_templates::Template;
use rocket::{http::Header, fairing::{Fairing, Info, Kind}};

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to dashboard");
    Redirect::to("/")
}

#[catch(401)]
pub fn catch_unauthorized() -> Redirect {
    log::info!("Unauthorized access detected - redirecting to login");
    Redirect::to("/")
}

/// CORS fairing for PWA functionality
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers",
            kind: Kind::Response
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "POST, GET, PATCH, OPTIONS"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

/// Work Time-specific service registry
pub struct WorkTimeServiceRegistry;

impl WorkTimeServiceRegistry {
    pub fn attach_all_services(rocket: Rocket<Build>) -> Rocket<Build> {
        log::info!("Registering work time application services...");
        
        log::debug!("Attaching work time services: Auth, WorkTime, PayPeriod, Settings");
        
        rocket
            .manage(AuthService::new())
            .manage(WorkTimeService::new())
            .manage(PayPeriodService::new())
            .manage(SettingsService::new())
    }
}

/// Work Time-specific controller registry  
pub struct WorkTimeControllerRegistry;

impl WorkTimeControllerRegistry {
    pub fn attach_all_controllers(rocket: Rocket<Build>) -> Rocket<Build> {
        log::info!("Registering work time application controllers...");
        log::debug!("Attaching controllers: WorkTime (/)");
        
        rocket
            .attach(controllers::WorkTimeController::new("/".to_owned()))
    }
}





/// Offline page route
#[get("/offline.html")]
pub fn offline_page() -> (rocket::http::ContentType, String) {
    let offline_html = "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>Offline - Work Time Tracker</title>
    <style>
        body {
            font-family: 'Courier New', monospace;
            background-color: #1a1a1a;
            color: #00ff00;
            margin: 0;
            padding: 20px;
            text-align: center;
        }
        .container {
            max-width: 600px;
            margin: 50px auto;
        }
        .ascii-art {
            font-size: 12px;
            line-height: 1;
            margin: 20px 0;
        }
        .message {
            font-size: 18px;
            margin: 30px 0;
        }
        .retry-btn {
            background: #00ff00;
            color: #1a1a1a;
            border: none;
            padding: 10px 20px;
            font-family: inherit;
            font-size: 16px;
            cursor: pointer;
            margin: 10px;
        }
    </style>
</head>
<body>
    <div class=\"container\">
        <div class=\"ascii-art\">
⚠️ OFFLINE MODE ⚠️
        </div>
        <div class=\"message\">
            <h1>You're currently offline</h1>
            <p>Work Time Tracker is not available right now.</p>
            <p>Please check your internet connection and try again.</p>
        </div>
        <button class=\"retry-btn\" onclick=\"window.location.reload()\">Retry Connection</button>
        <button class=\"retry-btn\" onclick=\"window.location.href='/'\">Go to Dashboard</button>
    </div>
</body>
</html>".to_string();
    (rocket::http::ContentType::HTML, offline_html)
}

#[launch]
async fn rocket() -> Rocket<Build> {
    log::info!("Starting Work Time Tracker PWA application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Log level: {:?}", Features::log_level());
    
    // Build the base rocket instance
    log::info!("Building Work Time Tracker Rocket instance and attaching services...");
    let mut rocket = create_base_rocket()
        .register("/", catchers![catch_default, catch_unauthorized])
        .attach(Template::fairing())
        .attach(CORS);
    
    // Attach work time-specific services
    rocket = WorkTimeServiceRegistry::attach_all_services(rocket);
    
    // Attach work time controllers and static file server
    WorkTimeControllerRegistry::attach_all_controllers(rocket)
        .mount("/", rocket::routes![offline_page])
        .mount("/static", FileServer::from("./static/"))
}