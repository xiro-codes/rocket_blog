//! Business logic and data access services.
//!
//! This module contains the core business logic of the application, organized
//! into services that handle specific domains like authentication, blog posts,
//! comments, and AI integration.
//!
//! ## Service Architecture
//!
//! Services follow a layered architecture pattern:
//! - **Base Services**: Common functionality and database operations
//! - **Domain Services**: Specific business logic for each feature area
//! - **Integration Services**: External API and AI provider integrations
//! - **Coordinator Services**: Orchestrate multiple services for complex operations
//!
//! ## Service Categories
//!
//! ### Core Services
//! - [`AuthService`] - User authentication and session management
//! - [`BlogService`] - Blog post CRUD operations and publishing
//! - [`CommentService`] - Comment management and moderation
//! - [`SettingsService`] - Application configuration management
//!
//! ### AI Integration Services  
//! - [`AIProviderService`] - AI provider coordination and management
//! - [`OpenAIService`] - OpenAI API integration for content generation
//! - [`OllamaService`] - Local Ollama model integration
//!
//! ### Specialized Services
//! - [`WorkTimeService`] - Time tracking and work period management
//! - [`ReactionService`] - Post reactions and engagement tracking
//! - [`TagService`] - Tag management and categorization
//! - [`BackgroundJobService`] - Asynchronous task processing

mod base;
pub use base::{BaseService, ServiceHelpers, ManagedService, CrudService};

mod coordinator;
/// Service coordination and complex operation orchestration
pub use coordinator::{CoordinatorService, BlogListData, BlogDetailData, BlogSearchData, BlogTagData};

mod auth;
/// User authentication and session management service
pub use auth::Service as AuthService;

mod background_job;
/// Background task processing and job queue management
pub use background_job::BackgroundJobService;

mod blog;
/// Blog post management and publishing service
pub use blog::Service as BlogService;

mod comment;
/// Comment system and moderation service
pub use comment::Service as CommentService;

mod ai_provider;
/// AI provider abstraction and management service
pub use ai_provider::{AIProvider, AIProviderService};

mod openai;
/// OpenAI API integration service
pub use openai::OpenAIService;

mod ollama;
/// Local Ollama model integration service
pub use ollama::OllamaService;

mod reaction;
/// Post reaction and engagement tracking service
pub use reaction::{PostReactionSummary, Service as ReactionService};

mod settings;
/// Application settings and configuration service
pub use settings::SettingsService;

mod tag;
pub use tag::TagService;
mod timezone;
/// Timezone conversion and user preference service
pub use timezone::TimezoneService;
mod work_time;
pub use work_time::WorkTimeService;
mod pay_period;
pub use pay_period::PayPeriodService;
mod youtube;
pub use youtube::YoutubeDownloadService;
