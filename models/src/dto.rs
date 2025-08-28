//! Data Transfer Objects and Form structures for the blog application.
//!
//! This module contains custom structs that are used alongside the generated
//! SeaORM entities but are not part of the database schema generation.

use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;
use sea_orm::{DerivePartialModel, FromQueryResult};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

/// Form DTO for account authentication and registration
#[derive(
    Clone,
    Debug,
    PartialEq,
    DerivePartialModel,
    FromQueryResult,
    Eq,
    Serialize,
    Deserialize,
    FromForm,
)]
#[serde(crate = "rocket::serde")]
#[sea_orm(entity = "super::account::Entity")]
pub struct AccountFormDTO {
    pub username: String,
    pub password: String,
}

/// Form DTO for admin account creation with email
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct AdminCreateFormDTO {
    pub username: String,
    pub password: String,
    pub email: String,
}

/// Form DTO for comment creation
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct CommentFormDTO {
    pub text: String,
    pub username: Option<String>, // For anonymous users
    pub parent_id: Option<Uuid>, // For threaded replies
}

/// Result struct for post title queries
#[derive(
    Clone, Debug, PartialEq, Eq, DerivePartialModel, FromQueryResult, Serialize, Deserialize,
)]
#[serde(crate = "rocket::serde")]
#[sea_orm(entity = "super::post::Entity")]
pub struct PostTitleResult {
    pub id: Uuid,
    pub seq_id: i32,
    pub title: String,
    pub draft: Option<bool>,
    pub excerpt: Option<String>,
}

/// Result struct for search results with ranking
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct PostSearchResult {
    pub id: Uuid,
    pub seq_id: i32,
    pub title: String,
    pub excerpt: Option<String>,
    pub rank: f32,
    pub headline: Option<String>,
}

/// Search form for handling search queries
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct SearchFormDTO {
    pub query: String,
}

/// Form DTO for settings configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct SettingsFormDTO {
    pub openai_api_key: String,
}

/// Form DTO for Ollama settings configuration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct OllamaSettingsFormDTO {
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_enabled: bool,
}

/// Form DTO for creating user roles
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct UserRoleFormDTO {
    pub role_name: String,
    pub hourly_wage: String, // Use String to handle form input, convert to Decimal in service
    pub currency: String,
}

/// Form DTO for creating work time entries
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct WorkTimeEntryFormDTO {
    pub user_role_id: Uuid,
    pub start_time: Option<String>, // Use String for form handling
    pub end_time: Option<String>,
    pub description: Option<String>,
    pub project: Option<String>,
}

/// Form DTO for time tracking controls (start/stop)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct TimeTrackingControlDTO {
    pub user_role_id: Uuid,
    pub description: Option<String>,
    pub project: Option<String>,
}

/// Result struct for work time summary queries
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct WorkTimeSummaryDTO {
    pub total_hours: Decimal,
    pub total_earnings: Decimal,
    pub currency: String,
    pub entries_count: i32,
}

/// Result struct for work time entry with role information
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct WorkTimeEntryWithRoleDTO {
    pub id: Uuid,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub description: Option<String>,
    pub project: Option<String>,
    pub is_active: bool,
    pub role_name: String,
    pub hourly_wage: Decimal,
    pub currency: String,
    pub earnings: Option<Decimal>,
}
