pub mod post {
    use rocket::FromForm;
    use rocket::fs::TempFile;

    #[derive(FromForm)]
    pub struct FormDTO<'r> {
        pub title: String,
        pub text: String,
        pub file: TempFile<'r>,
    }
}
