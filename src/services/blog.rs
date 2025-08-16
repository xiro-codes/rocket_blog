use crate::{config::AppConfig, dto::post::FormDTO};
use chrono::Local;
use models::{
    account,
    dto::PostTitleResult,
    post,
    post_tag,
    prelude::{Account, Post, Tag},
    tag,
};
use rocket::State;
use sea_orm::{ColumnTrait, JoinType, *};
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

    /// Generate an excerpt from the text content if no excerpt is provided
    fn generate_excerpt(text: &str, provided_excerpt: Option<String>) -> Option<String> {
        if let Some(excerpt) = provided_excerpt {
            if !excerpt.trim().is_empty() {
                return Some(excerpt.trim().to_string());
            }
        }
        
        // Remove markdown formatting and HTML tags for a clean excerpt
        let clean_text = text
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect::<Vec<&str>>()
            .join(" ");
        
        // Take first 200 characters and try to end at a word boundary
        if clean_text.len() <= 200 {
            Some(clean_text)
        } else {
            let truncated = &clean_text[..200];
            if let Some(last_space) = truncated.rfind(' ') {
                Some(format!("{}...", &truncated[..last_space]))
            } else {
                Some(format!("{}...", truncated))
            }
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
        let excerpt = Self::generate_excerpt(&data.text, data.excerpt.clone());
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
            excerpt: Set(excerpt),
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
        let excerpt = Self::generate_excerpt(&data.text, data.excerpt);
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.excerpt = Set(excerpt);
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
        let excerpt = Self::generate_excerpt(&data.text, data.excerpt);
        p.title = Set(data.title.to_owned());
        p.text = Set(data.text.to_owned());
        p.excerpt = Set(excerpt);
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
            .column(post::Column::Draft)
            .column(post::Column::Excerpt)
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
        self.paginate_with_title_include_drafts(db, page, page_size, false).await
    }

    pub async fn paginate_with_title_include_drafts(
        &self,
        db: &DbConn,
        page: Option<u64>,
        page_size: Option<u64>,
        include_drafts: bool,
    ) -> Result<(Vec<PostTitleResult>, u64, u64, u64), DbErr> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
        let mut query = Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .column(post::Column::Draft);
        
        if !include_drafts {
            query = query.filter(post::Column::Draft.eq(false));
        }
        
        let paginator = query
            .column(post::Column::Excerpt)
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
    
    pub async fn paginate_posts_by_tag(
        &self,
        db: &DbConn,
        tag_id: uuid::Uuid,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(Vec<PostTitleResult>, u64, u64, u64), DbErr> {
        self.paginate_posts_by_tag_include_drafts(db, tag_id, page, page_size, false).await
    }
    
    pub async fn paginate_posts_by_tag_include_drafts(
        &self,
        db: &DbConn,
        tag_id: uuid::Uuid,
        page: Option<u64>,
        page_size: Option<u64>,
        include_drafts: bool,
    ) -> Result<(Vec<PostTitleResult>, u64, u64, u64), DbErr> {
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
        
        // Join posts with post_tag to filter by tag_id
        let mut query = Post::find()
            .select_only()
            .column(post::Column::Id)
            .column(post::Column::Title)
            .column(post::Column::SeqId)
            .column(post::Column::Draft)
            .column(post::Column::Excerpt)
            .join(JoinType::InnerJoin, post::Relation::PostTag.def())
            .filter(post_tag::Column::TagId.eq(tag_id));
            
        if !include_drafts {
            query = query.filter(post::Column::Draft.eq(false));
        }
        
        let paginator = query
            .order_by_desc(post::Column::DatePublished)
            .into_partial_model()
            .paginate(db, page_size);
        let num_pages = paginator.num_pages().await?;
        paginator
            .fetch_page(page - 1)
            .await
            .map(|p| (p, page, page_size, num_pages))
    }

    /// Fetch recent published posts for RSS feed
    pub async fn find_recent_published_posts(
        &self,
        db: &DbConn,
        limit: Option<u64>,
    ) -> Result<Vec<post::Model>, DbErr> {
        let limit = limit.unwrap_or(20); // Default to 20 recent posts
        
        Post::find()
            .filter(post::Column::Draft.eq(false))
            .order_by_desc(post::Column::DatePublished)
            .limit(limit)
            .all(db)
            .await
    }

    /// Search posts using PostgreSQL full-text search
    pub async fn search_posts(
        &self,
        db: &DbConn,
        query: &str,
        include_drafts: bool,
        page: Option<u64>,
        page_size: Option<u64>,
    ) -> Result<(Vec<models::dto::PostSearchResult>, u64, u64, u64), DbErr> {
        use models::dto::PostSearchResult;
        use sea_orm::Statement;
        
        let page = page.unwrap_or(1);
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        if page == 0 {
            return Err(DbErr::Custom("Page number cannot be zero".to_owned()));
        }
        if page_size == 0 {
            return Err(DbErr::Custom("Page size cannot be zero".to_owned()));
        }
        
        if query.trim().is_empty() {
            return Ok((vec![], 0, page_size, 0));
        }

        let offset = (page - 1) * page_size;
        
        // Sanitize the query - escape special characters and prepare for tsquery
        let tsquery = Self::prepare_tsquery(query);
        
        // Build the search SQL with ranking and headline generation
        let draft_filter = if include_drafts {
            ""
        } else {
            "AND (draft = false OR draft IS NULL)"
        };
        
        let search_sql = format!(
            r#"
            SELECT 
                p.id,
                p.seq_id,
                p.title,
                p.excerpt,
                ts_rank_cd(p.search_vector, to_tsquery('english', $1)) as rank,
                ts_headline('english', 
                    COALESCE(p.title, '') || ' ' || COALESCE(p.text, '') || ' ' || COALESCE(p.excerpt, ''),
                    to_tsquery('english', $1),
                    'StartSel=<mark>, StopSel=</mark>, MaxWords=50, MinWords=10'
                ) as headline
            FROM post p 
            WHERE p.search_vector @@ to_tsquery('english', $1)
            {}
            ORDER BY rank DESC, p.date_published DESC
            LIMIT $2 OFFSET $3
            "#,
            draft_filter
        );
        
        let count_sql = format!(
            r#"
            SELECT COUNT(*) as count
            FROM post p 
            WHERE p.search_vector @@ to_tsquery('english', $1)
            {}
            "#,
            draft_filter
        );

        // Execute count query first
        let count_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            &count_sql,
            vec![tsquery.clone().into()]
        );
        
        let count_result: Option<sea_orm::QueryResult> = db.query_one(count_stmt).await?;
        let total_count = if let Some(row) = count_result {
            row.try_get::<i64>("", "count")? as u64
        } else {
            0
        };
        
        let num_pages = (total_count + page_size - 1) / page_size;

        // Execute search query
        let search_stmt = Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Postgres,
            &search_sql,
            vec![
                tsquery.into(),
                (page_size as i64).into(),
                (offset as i64).into()
            ]
        );
        
        let search_results = db.query_all(search_stmt).await?;
        
        // Convert results to PostSearchResult structs
        let mut results = Vec::new();
        for row in search_results {
            let result = PostSearchResult {
                id: row.try_get("", "id")?,
                seq_id: row.try_get("", "seq_id")?,
                title: row.try_get("", "title")?,
                excerpt: row.try_get("", "excerpt").ok(),
                rank: row.try_get("", "rank")?,
                headline: row.try_get("", "headline").ok(),
            };
            results.push(result);
        }

        Ok((results, page, page_size, num_pages))
    }

    /// Prepare a search query for PostgreSQL tsquery format
    fn prepare_tsquery(query: &str) -> String {
        // Split by whitespace and clean each term
        let terms: Vec<String> = query
            .split_whitespace()
            .map(|term| {
                // Remove special characters that could break tsquery, but keep basic ones
                let cleaned = term
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
                    .collect::<String>();
                
                if cleaned.is_empty() {
                    String::new()
                } else {
                    // Add prefix matching for partial word searches
                    format!("{}:*", cleaned)
                }
            })
            .filter(|term| !term.is_empty())
            .collect();

        if terms.is_empty() {
            // Fallback for empty query
            String::from("''")
        } else {
            // Join terms with AND operator
            terms.join(" & ")
        }
    }
}
