use app::{
    controllers::{
        AuthController, BlogController, CommentController, FeedController, HandymanController,
        IndexController, PortfolioController, SeoController, WorkTimeApiController,
        WorkTimeController,
    },
    create_base_rocket_with_database,
    database::parse_database_args_with_fallback,
    features::Features,
    middleware,
    services::{
        AuthService, BackgroundJobService, BlogService, CommentService, CoordinatorService,
        PayPeriodService, ReactionService, SettingsService, TagService, WorkTimeService,
        YoutubeDownloadService,
    },
    template_config,
};
use rocket::{
    catch, catchers,
    fairing::{Fairing, Info, Kind},
    fs::FileServer,
    get,
    http::Header,
    launch,
    response::Redirect,
    Build, Request, Response, Rocket,
};

#[catch(default)]
pub fn catch_default() -> Redirect {
    log::warn!("Unhandled route accessed - redirecting to home page");
    Redirect::to("/")
}

#[catch(401)]
pub fn catch_unauthorized() -> Redirect {
    log::info!("Unauthorized access detected - redirecting to home");
    Redirect::to("/")
}

/// CORS fairing for PWA functionality
pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}

/// Offline page route for Work Time Tracker PWA
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
</html>"
        .to_string();
    (rocket::http::ContentType::HTML, offline_html)
}

#[launch]
async fn rocket() -> Rocket<Build> {
    log::info!("Starting Rocket Web Application...");
    log::debug!("Development mode: {}", Features::is_development());
    log::debug!("Seeding enabled: {}", Features::enable_seeding());
    log::debug!("Log level: {:?}", Features::log_level());

    // Parse command line arguments for database configuration
    let db_config = parse_database_args_with_fallback();
    log::info!("Database configuration: {:?}", db_config);

    // Build the base rocket instance with database auto-detection
    log::info!("Building Rocket instance and configuring database...");
    let mut rocket = create_base_rocket_with_database(db_config)
        .await
        .register("/", catchers![catch_default, catch_unauthorized])
        .attach(template_config::create_template_fairing())
        .attach(CORS);

    // Attach all services
    rocket = rocket
        .manage(AuthService::new())
        .manage(BlogService::new())
        .manage(CommentService::new())
        .manage(ReactionService::new())
        .manage(SettingsService::new())
        .manage(TagService::new())
        .manage(CoordinatorService::new())
        .manage(YoutubeDownloadService::new())
        .manage(BackgroundJobService::new())
        .manage(WorkTimeService::new())
        .manage(PayPeriodService::new());

    // Always attach seeding middleware to create default admin if needed
    let seed_count = if Features::enable_seeding() { 50 } else { 0 };
    log::info!(
        "Attaching database initialization middleware (seed count: {})",
        seed_count
    );
    rocket = rocket.attach(middleware::Seeding::new(Some(0), seed_count));

    log::info!("Attaching controllers and static file server");

    // Attach all controllers
    rocket = rocket
        .attach(AuthController::new("/auth".to_owned()))
        .attach(BlogController::new("/blog".to_owned()))
        .attach(CommentController::new("/comment".to_owned()))
        .attach(FeedController::new("/feed".to_owned()))
        .attach(SeoController::new("/".to_owned()))
        .attach(WorkTimeController::new("/worktime".to_owned()))
        .attach(WorkTimeApiController::new("/api/worktime".to_owned()))
        .attach(HandymanController::new("/handyman".to_owned()))
        .attach(PortfolioController::new("/".to_owned()));

    rocket
        .mount("/worktime", rocket::routes![offline_page])
        .mount("/static", FileServer::from("./static/"))
}
