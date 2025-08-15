pub mod post {
    use rocket::{fs::TempFile, FromForm};

    #[derive(FromForm, Debug)]
    pub struct FormDTO<'r> {
        pub title: String,
        pub text: String,
        pub excerpt: Option<String>,
        pub file: TempFile<'r>,
        pub tags: Option<String>,
    }
}
