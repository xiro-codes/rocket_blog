use crate::services::{AuthService, BlogService, CommentService, ReactionService, TagService};
use models::{dto::PostTitleResult, post::Model as Post, tag};
use sea_orm::{DatabaseConnection, ColumnTrait, EntityTrait, QueryFilter};
use uuid::Uuid;

/// Coordinator service that orchestrates operations across multiple services
/// This reduces the coupling between controllers and individual services
pub struct CoordinatorService {
    auth_service: AuthService,
    blog_service: BlogService,
    comment_service: CommentService,
    reaction_service: ReactionService,
    tag_service: TagService,
}

impl CoordinatorService {
    pub fn new() -> Self {
        Self {
            auth_service: AuthService::new(),
            blog_service: BlogService::new(),
            comment_service: CommentService::new(),
            reaction_service: ReactionService::new(),
            tag_service: TagService::new(),
        }
    }

    /// Get paginated blog posts with all associated data for the list view
    pub async fn get_blog_list_data(
        &self,
        db: &DatabaseConnection,
        page: Option<u64>,
        page_size: Option<u64>,
        token: Option<&str>,
        client_ip: &str,
    ) -> Result<BlogListData, String> {
        let page_num = page.unwrap_or(1);
        let size = page_size.unwrap_or(10);
        log::debug!("Coordinator: getting blog list data - page={}, size={}, client_ip={}", page_num, size, client_ip);
        
        // Check if any accounts exist for the admin creation button
        log::debug!("Coordinator: checking if accounts exist");
        let has_accounts = self.auth_service.has_any_accounts(db).await;
        
        // Check if user is admin to include drafts
        let is_admin = if let Some(token_str) = token {
            log::debug!("Coordinator: checking admin status for token");
            self.auth_service.is_admin_token(db, token_str).await
        } else {
            log::debug!("Coordinator: no token provided, treating as non-admin");
            false
        };
        
        if is_admin {
            log::debug!("Coordinator: user is admin, including draft posts");
        } else {
            log::debug!("Coordinator: user is not admin, excluding draft posts");
        }
        
        // Get paginated posts
        log::debug!("Coordinator: fetching paginated posts");
        let (posts, page, page_size, num_pages) = self.blog_service
            .paginate_with_title_include_drafts(db, page, page_size, is_admin)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to get posts: {}", e);
                log::error!("Coordinator: {}", error_msg);
                error_msg
            })?;
        
        // Get all tags for the tag cloud
        log::debug!("Coordinator: fetching all tags");
        let all_tags = self.tag_service.find_all_tags(db).await.unwrap_or_else(|e| {
            log::warn!("Coordinator: failed to fetch tags: {}", e);
            Vec::new()
        });
        
        // Get reaction summaries for all posts
        log::debug!("Coordinator: fetching reaction summaries for {} posts", posts.len());
        let post_ids: Vec<Uuid> = posts.iter().map(|p| p.id).collect();
        let reaction_summaries = self.reaction_service
            .get_posts_reaction_summaries(db, &post_ids, client_ip)
            .await
            .map(|hashmap| hashmap.into_values().collect())
            .unwrap_or_else(|e| {
                log::warn!("Coordinator: failed to fetch reaction summaries: {}", e);
                Vec::new()
            });
        
        log::info!("Coordinator: blog list data prepared - {} posts, {} tags, {} reaction summaries", 
                  posts.len(), all_tags.len(), reaction_summaries.len());
        
        Ok(BlogListData {
            posts,
            page,
            page_size,
            num_pages,
            all_tags,
            reaction_summaries,
            has_accounts,
        })
    }

    /// Get blog post detail with all associated data
    pub async fn get_blog_detail_data(
        &self,
        db: &DatabaseConnection,
        id: Uuid,
        token: Option<&str>,
        client_ip: &str,
    ) -> Result<BlogDetailData, String> {
        // Get the post
        let post = self.blog_service.find_by_id(db, id).await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Post not found".to_string())?;
        
        // Check if user is admin (for draft access)
        let is_admin = if let Some(token_str) = token {
            self.auth_service.is_admin_token(db, token_str).await
        } else {
            false
        };
        
        // Check if post is published or user is admin
        if post.draft.unwrap_or(false) && !is_admin {
            return Err("Post not found".to_string());
        }
        
        // Get tags for the post
        let tags = self.tag_service.find_tags_by_post_id(db, post.id).await
            .unwrap_or_default();
        
        // Get comments for the post
        let comments = self.comment_service.find_many_by_post_id(db, post.id).await
            .unwrap_or_default();
        
        // Get reaction summaries
        let reaction_summaries = self.reaction_service
            .get_posts_reaction_summaries(db, &[post.id], client_ip)
            .await
            .map(|hashmap| hashmap.into_values().collect())
            .unwrap_or_default();
        
        Ok(BlogDetailData {
            post,
            tags,
            comments,
            reaction_summaries,
        })
    }

    /// Search blog posts with all associated data
    pub async fn search_blog_posts(
        &self,
        db: &DatabaseConnection,
        query: &str,
        page: Option<u64>,
        page_size: Option<u64>,
        token: Option<&str>,
        client_ip: &str,
    ) -> Result<BlogSearchData, String> {
        // Check if user is admin to include drafts
        let is_admin = if let Some(token_str) = token {
            self.auth_service.is_admin_token(db, token_str).await
        } else {
            false
        };
        
        // Perform search
        let (results, page, page_size, num_pages) = if !query.trim().is_empty() {
            self.blog_service.search_posts(db, query, is_admin, page, page_size)
                .await
                .map_err(|e| format!("Search failed: {}", e))?
        } else {
            (vec![], page.unwrap_or(1), page_size.unwrap_or(10), 0)
        };
        
        // Get all tags for the tag cloud
        let all_tags = self.tag_service.find_all_tags(db).await.unwrap_or_default();
        
        // Get reaction summaries for search results if any
        let reaction_summaries = if !results.is_empty() {
            let post_ids: Vec<Uuid> = results.iter().map(|p| p.id).collect();
            self.reaction_service
                .get_posts_reaction_summaries(db, &post_ids, client_ip)
                .await
                .map(|hashmap| hashmap.into_values().collect())
                .unwrap_or_default()
        } else {
            vec![]
        };
        
        Ok(BlogSearchData {
            results,
            page,
            page_size,
            num_pages,
            all_tags,
            reaction_summaries,
        })
    }

    /// Get posts filtered by tag with all associated data
    pub async fn get_posts_by_tag_data(
        &self,
        db: &DatabaseConnection,
        slug: &str,
        page: Option<u64>,
        page_size: Option<u64>,
        token: Option<&str>,
        client_ip: &str,
    ) -> Result<BlogTagData, String> {
        let page_num = page.unwrap_or(1);
        let size = page_size.unwrap_or(10);
        log::debug!("Coordinator: getting posts by tag data - slug={}, page={}, size={}, client_ip={}", slug, page_num, size, client_ip);
        let has_accounts = self.auth_service.has_any_accounts(db).await;

        // Check if user is admin to include drafts
        let is_admin = if let Some(token_str) = token {
            log::debug!("Coordinator: checking admin status for token");
            self.auth_service.is_admin_token(db, token_str).await
        } else {
            log::debug!("Coordinator: no token provided, treating as non-admin");
            false
        };

        if is_admin {
            log::debug!("Coordinator: user is admin, including draft posts");
        } else {
            log::debug!("Coordinator: user is not admin, excluding draft posts");
        }

        // Find tag by slug
        log::debug!("Coordinator: finding tag by slug: {}", slug);
        let tag = tag::Entity::find()
            .filter(tag::Column::Slug.eq(slug))
            .one(db)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to find tag: {}", e);
                log::error!("Coordinator: {}", error_msg);
                error_msg
            })?
            .ok_or_else(|| {
                let error_msg = format!("Tag not found: {}", slug);
                log::error!("Coordinator: {}", error_msg);
                error_msg
            })?;

        // Get posts with this tag
        log::debug!("Coordinator: fetching posts for tag: {}", tag.name);
        let (posts, page, page_size, num_pages) = self.blog_service
            .paginate_posts_by_tag_include_drafts(db, tag.id, Some(page_num), Some(size), is_admin)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to get posts by tag: {}", e);
                log::error!("Coordinator: {}", error_msg);
                error_msg
            })?;

        // Get all tags for the tag cloud
        log::debug!("Coordinator: fetching all tags");
        let all_tags = self.tag_service.find_all_tags(db).await.unwrap_or_else(|e| {
            log::warn!("Coordinator: failed to fetch tags: {}", e);
            Vec::new()
        });

        // Get reaction summaries for all posts
        log::debug!("Coordinator: fetching reaction summaries for {} posts", posts.len());
        let post_ids: Vec<Uuid> = posts.iter().map(|p| p.id).collect();
        let reaction_summaries = self.reaction_service
            .get_posts_reaction_summaries(db, &post_ids, client_ip)
            .await
            .map(|hashmap| hashmap.into_values().collect())
            .unwrap_or_else(|e| {
                log::warn!("Coordinator: failed to fetch reaction summaries: {}", e);
                Vec::new()
            });

        log::debug!("Coordinator: successfully fetched tag data for {} posts", posts.len());
        
        Ok(BlogTagData {
            posts,
            page,
            page_size,
            num_pages,
            all_tags,
            reaction_summaries,
            tag,
            has_accounts,
        })
    }

    /// Check if token belongs to an admin user
    pub async fn is_admin(&self, db: &DatabaseConnection, token: Option<&str>) -> bool {
        if let Some(token_str) = token {
            self.auth_service.is_admin_token(db, token_str).await
        } else {
            false
        }
    }

    /// Require admin access and return account
    pub async fn require_admin(&self, db: &DatabaseConnection, token: Option<&str>) -> Result<models::account::Model, String> {
        let token_str = token.ok_or("Authentication required")?;
        self.auth_service.require_admin_token(db, token_str).await
            .map_err(|e| format!("Admin access required: {}", e))
    }
}

/// Data structure for blog list view
pub struct BlogListData {
    pub posts: Vec<PostTitleResult>,
    pub page: u64,
    pub page_size: u64,
    pub num_pages: u64,
    pub all_tags: Vec<models::tag::Model>,
    pub reaction_summaries: Vec<crate::services::PostReactionSummary>,
    pub has_accounts: bool,
}

/// Data structure for blog detail view  
pub struct BlogDetailData {
    pub post: Post,
    pub tags: Vec<models::tag::Model>,
    pub comments: Vec<models::comment::Model>,
    pub reaction_summaries: Vec<crate::services::PostReactionSummary>,
}

/// Data structure for blog search results
pub struct BlogSearchData {
    pub results: Vec<models::dto::PostSearchResult>,
    pub page: u64,
    pub page_size: u64,
    pub num_pages: u64,
    pub all_tags: Vec<models::tag::Model>,
    pub reaction_summaries: Vec<crate::services::PostReactionSummary>,
}

/// Data structure for blog tag filter results
pub struct BlogTagData {
    pub posts: Vec<models::dto::PostTitleResult>,
    pub page: u64,
    pub page_size: u64,
    pub num_pages: u64,
    pub all_tags: Vec<models::tag::Model>,
    pub reaction_summaries: Vec<crate::services::PostReactionSummary>,
    pub tag: models::tag::Model,
    pub has_accounts: bool,
}
