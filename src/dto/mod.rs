pub mod post {
    use rocket::FromForm;
    use rocket::fs::TempFile;

    #[derive(FromForm, Debug)]
    pub struct FormDTO<'r> {
        pub title: String,
        pub text: String,
        pub file: TempFile<'r>,
        pub tags: Option<String>,
    }
}
