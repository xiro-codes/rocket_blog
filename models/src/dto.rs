//! Data Transfer Objects and Form structures for the blog application.
//! 
//! This module contains custom structs that are used alongside the generated
//! SeaORM entities but are not part of the database schema generation.

use sea_orm::{DerivePartialModel, FromQueryResult};
use rocket::serde::{Deserialize, Serialize};
use rocket::FromForm;
use uuid::Uuid;

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

/// Form DTO for comment creation
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    FromForm,
)]
#[serde(crate = "rocket::serde")]
pub struct CommentFormDTO {
    pub text: String,
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
}