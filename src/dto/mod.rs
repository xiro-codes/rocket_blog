pub mod post {
    use rocket::{fs::TempFile, FromForm};

    #[derive(FromForm, Debug)]
    pub struct FormDTO<'r> {
        pub title: String,
        pub text: String,
        pub excerpt: Option<String>,
        pub file: TempFile<'r>,
        pub tags: Option<String>,
        pub action: Option<String>, // "draft" or "publish"
        pub ai_generate: Option<String>, // "content", "excerpt", "tags"
        pub ai_prompt: Option<String>, // Additional prompt for AI generation
    }
}
