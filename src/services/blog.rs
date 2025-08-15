use crate::{config::AppConfig, dto::post::FormDTO};
use chrono::Local;
use models::{
    account,
    dto::PostTitleResult,
    post,
    prelude::{Account, Post, Tag},
    tag,
};
use rocket::State;
use sea_orm::{ColumnTrait, *};
use uuid::Uuid;

use crate::services::base::BaseService;

pub struct Service {
    base: BaseService,
}

const DEFAULT_PAGE_SIZE: u64 = 39;

impl Service {
    pub fn new() -> Self {
        Self {
            base: BaseService::new(),
        }
    }

    pub async fn create(
        &self,
        db: &DbConn,
        app_config: &State<AppConfig>,
        id: Uuid,
        data: &mut FormDTO<'_>,
    ) -> Result<post::Model, DbErr> {
        let text = markdown::to_html(data.text.as_str());
        let fid = BaseService::generate_id().to_string();
        let path = if let Some(name) = data.file.name() {
            let path = format!("{}/{}_{}.webm", app_config.data_path, fid, name);
            data.file
                .copy_to(path.clone())
                .await
                .map_err(|e| DbErr::Custom(e.to_string()))?;
            Some(path)
        } else {
            None
        };

        post::ActiveModel {
            id: Set(BaseService::generate_id()),
            title: Set(data.title.to_owned()),
            text: Set(text),
            path: Set(path),
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

    pub async fn delete_by_id(&self, _db: &DbConn, _id: Uuid) -> Result<DeleteResult, DbErr> {
        todo!()
    }

    pub async fn delete_by_seq_id(&self, db: &DbConn, id: i32) -> Result<(), DbErr> {
        let mut p = self.find_by_seq_id(db, id).await?.into_active_model();
        p.draft = Set(Some(true));
        p.save(db).await.map(|_| ())
    }

    pub async fn find_by_id(&self, db: &DbConn, id: Uuid) -> Result<Option<post::Model>, DbErr> {
        Post::find_by_id(id).one(db).await
    }

    pub async fn find_by_seq_id(&self, db: &DbConn, id: i32) -> Result<post::Model, DbErr> {
        let result = Post::find()
            .filter(post::Column::SeqId.eq(id))
            .one(db)
            .await?;
        BaseService::handle_not_found(result, "Post")
    }

    pub async fn find_by_seq_id_with_account(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<(post::Model, Option<account::Model>), DbErr> {
        let result = Post::find()
            .filter(post::Column::SeqId.eq(id))
            .find_also_related(Account)
            .one(db)
            .await?;
        BaseService::handle_not_found(result, "Post")
    }
    pub async fn find_by_seq_id_with_account_and_tags(
        &self,
        db: &DbConn,
        id: i32,
    ) -> Result<(post::Model, Option<account::Model>, Option<tag::Model>), DbErr> {
        let result = Post::find()
            .filter(post::Column::SeqId.eq(id))
            .find_also_related(Account)
            .find_also_related(Tag)
            .one(db)
            .await?;
        BaseService::handle_not_found(result, "Post")
    }
    pub async fn find_many_with_title(&self, db: &DbConn) -> Result<Vec<PostTitleResult>, DbErr> {
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
    ) -> Result<(Vec<PostTitleResult>, u64, u64, u64), DbErr> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
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
