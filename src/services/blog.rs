use crate::{dto::post::FormDTO, generic::{CrudService, PaginationUtils, ErrorUtils}};
use chrono::Local;
use models::prelude::{Account, Post};
use models::{account, post};
use sea_orm::ColumnTrait;
use sea_orm::*;
use uuid::Uuid;

pub struct Service;
const DATA_PATH: &str = "/home/tod/.local/share/blog";

impl Service {
    pub fn new() -> Self {
        Self
    }
    
    /// Create with account ID - blog-specific method
    pub async fn create(
        &self,
        db: &DbConn,
        id: Uuid,
        data: &mut FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let text = markdown::to_html(data.text.as_str());
        let fid = Uuid::new_v4().to_string();
        let path = format!("{DATA_PATH}/{}_{}.webm", fid, data.file.name().unwrap());

        data.file
            .copy_to(path.clone())
            .await
            .map_err(|e| DbErr::Custom(e.to_string()))?;

        post::ActiveModel {
            id: Set(Uuid::new_v4()),
            title: Set(data.title.to_owned()),
            text: Set(text),
            path: Set(Some(path)),
            draft: Set(Some(true)),
            date_published: Set(Local::now().naive_local()),
            account_id: Set(id),
            ..Default::default()
        }
        .insert(db)
        .await
    }

    pub async fn update_by_id(
        &self,
        db: &DbConn,
        id: Uuid,
        data: FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let mut p: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)?;
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.update(db).await
    }
    pub async fn update_by_seq_id(
        &self,
        db: &DbConn,
        id: i32,
        data: FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let mut p: post::ActiveModel = Post::find()
            .filter(post::Column::SeqId.eq(id))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))
            .map(Into::into)?;
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.update(db).await
    }

    pub async fn delete_by_id(&self, db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        let mut post = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound(format!("Post with id: {}", id)))?
            .into_active_model();
        post.draft = Set(Some(true));
        post.save(db).await.map(|_| ())
    }
    pub async fn delete_by_seq_id(&self, db: &DbConn, id: i32) -> Result<(), DbErr> {
        let mut p = self.find_by_seq_id(db, id).await?.into_active_model();
        p.draft = Set(Some(true));
        p.save(db).await.map(|_|())
    }

    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }
    pub async fn find_by_seq_id(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<post::Model , DbErr> {
        Post::find()
            .filter(post::Column::SeqId.eq(id))
            .one(db)
            .await?
            .ok_or_else(|| ErrorUtils::not_found("Post", id))
    }

    pub async fn find_by_seq_id_with_account(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<(post::Model, Option<account::Model>), DbErr> {
        Post::find()
            .filter(post::Column::SeqId.eq(id))
            .find_also_related(Account)
            .one(db)
            .await?
            .ok_or_else(|| ErrorUtils::not_found("Post", id))
    }
    pub async fn find_many_with_title(&self, db: &DbConn) -> Result<Vec<post::TitleResult>, DbErr> {
        Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .into_partial_model()
            .all(db)
            .await
    }
    pub async fn find_mm_seq_id(&self, db: &DbConn) -> Result<Option<(i32, i32)>, DbErr> {
        Post::find()
            .select_only()
            .column_as(post::Column::SeqId.min(), "min_post")
            .column_as(post::Column::SeqId.max(), "max_post")
            .into_tuple::<(i32, i32)>()
            .one(db)
            .await
    }
    pub async fn paginate_with_title(
        &self,
        db: &DbConn,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(Vec<post::TitleResult>, u64, u64, u64), DbErr> {
        let (page, page_size) = PaginationUtils::normalize_pagination(page, page_size)?;
        
        let paginator = Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .filter(post::Column::Draft.eq(false))
            .order_by_desc(post::Column::DatePublished)
            .into_partial_model()
            .paginate(db, page_size);
        let num_pages = paginator.num_pages().await?;
        paginator
            .fetch_page(page - 1)
            .await
            .map(|p| (p, page, page_size, num_pages))
    }
}

// Implement generic CRUD operations for blog posts
impl CrudService<post::Model, FormDTO<'_>, FormDTO<'_>, Uuid> for Service {
    async fn create(&self, _db: &DbConn, _data: FormDTO<'_>) -> Result<post::Model, DbErr> {
        // Blog posts need account ID, so this generic method isn't directly used
        // Use the specific create method instead
        Err(DbErr::Custom("Use create with account ID instead".to_owned()))
    }
    
    async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }
    
    async fn update_by_id(&self, db: &DbConn, id: Uuid, data: FormDTO<'_>) -> Result<post::Model, DbErr> {
        let mut p: post::ActiveModel = Post::find_by_id(id)
            .one(db)
            .await?
            .ok_or_else(|| ErrorUtils::not_found("Post", id))
            .map(Into::into)?;
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.update(db).await
    }
    
    async fn delete_by_id(&self, db: &DbConn, id: Uuid) -> Result<(), DbErr> {
        self.delete_by_id(db, id).await
    }
}
