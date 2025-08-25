// Use shared base service instead of local one
pub use common::services::BaseService;

mod coordinator;
pub use coordinator::{CoordinatorService, BlogListData, BlogDetailData, BlogSearchData, BlogTagData};

// Use shared auth service instead of local one
pub use common::auth::AuthService;

mod background_job;
pub use background_job::BackgroundJobService;
mod blog;
pub use blog::Service as BlogService;
mod comment;
pub use comment::Service as CommentService;
mod ai_provider;
pub use ai_provider::{AIProvider, AIProviderService};
mod openai;
pub use openai::OpenAIService;
mod ollama;
pub use ollama::OllamaService;
mod reaction;
pub use reaction::{PostReactionSummary, Service as ReactionService};
mod settings;
pub use settings::SettingsService;
mod tag;
pub use tag::TagService;
mod youtube;
pub use youtube::YoutubeDownloadService;
