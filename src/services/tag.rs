use chrono::Local;
use models::{post_tag, tag};
use sea_orm::*;
use slug::slugify;
use uuid::Uuid;

use crate::services::base::BaseService;

pub struct TagService;

impl TagService {
    pub fn new() -> Self {
        Self
    }

    pub async fn create_tag(
        &self,
        db: &DbConn,
        name: &str,
        color: Option<String>,
    ) -> Result<tag::Model, DbErr> {
        let slug = slugify(name);

        tag::ActiveModel {
            id: Set(BaseService::generate_id()),
            name: Set(name.to_owned()),
            slug: Set(slug),
            color: Set(Some(color.unwrap_or("#007bff".to_string()))),
            created_at: Set(Local::now().naive_local()),
        }
        .insert(db)
        .await
    }

    pub async fn find_all_tags(&self, db: &DbConn) -> Result<Vec<tag::Model>, DbErr> {
        tag::Entity::find()
            .order_by_asc(tag::Column::Name)
            .all(db)
            .await
    }

    pub async fn find_tags_by_post_id(
        &self,
        db: &DbConn,
        post_id: Uuid,
    ) -> Result<Vec<tag::Model>, DbErr> {
        tag::Entity::find()
            .inner_join(post_tag::Entity)
            .filter(post_tag::Column::PostId.eq(post_id))
            .order_by_asc(tag::Column::Name)
            .all(db)
            .await
    }

    pub async fn add_tag_to_post(
        &self,
        db: &DbConn,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<post_tag::Model, DbErr> {
        post_tag::ActiveModel {
            post_id: Set(post_id),
            tag_id: Set(tag_id),
        }
        .insert(db)
        .await
    }

    pub async fn remove_tag_from_post(
        &self,
        db: &DbConn,
        post_id: Uuid,
        tag_id: Uuid,
    ) -> Result<DeleteResult, DbErr> {
        post_tag::Entity::delete_many()
            .filter(
                Condition::all()
                    .add(post_tag::Column::PostId.eq(post_id))
                    .add(post_tag::Column::TagId.eq(tag_id)),
            )
            .exec(db)
            .await
    }

    pub async fn find_or_create_tag(&self, db: &DbConn, name: &str) -> Result<tag::Model, DbErr> {
        let slug = slugify(name);

        if let Ok(existing_tag) = tag::Entity::find()
            .filter(tag::Column::Slug.eq(&slug))
            .one(db)
            .await
        {
            if let Some(tag) = existing_tag {
                return Ok(tag);
            }
        }

        self.create_tag(db, name, None).await
    }
}
