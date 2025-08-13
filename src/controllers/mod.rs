mod index {
    use rocket::{
        fairing::{self, Fairing, Info, Kind},
        Build, Rocket,
    };

    pub struct Controller {
        pub path: String,
    }
    impl Controller {
        pub fn new(path: String) -> Self {
            Self { path }
        }
    }
    #[get("/")]
    pub async fn index() -> &'static str {
        "Hello, world!"
    }

    #[rocket::async_trait]
    impl Fairing for Controller {
        fn info(&self) -> Info {
            Info {
                name: "Index Controller",
                kind: Kind::Ignite,
            }
        }
        async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
            Ok(rocket.mount(&self.path, routes![index]))
        }
    }
}
pub use index::Controller as IndexController;
