# Rocket Blog - New Feature Suggestions

## Overview
This document outlines potential new features for the Rocket Blog application, organized by implementation complexity and priority. All suggestions are designed to build upon the existing architecture with minimal breaking changes.

## Current Architecture Analysis

### Existing Features
- Blog post CRUD operations with markdown support
- User authentication and authorization (admin system)
- Comment system
- Video file streaming with range requests
- Pagination support
- Draft/published post states
- Template-based UI with Tera

### Database Schema
- `Account` - User management with admin privileges
- `Post` - Blog posts with sequential IDs, draft status, and file paths
- `Comment` - Comments linked to posts
- `Event` - Event tracking (currently minimal)

---

## Feature Suggestions by Priority

### 🟢 High Priority - Quick Wins (1-3 days implementation)

#### 1. Tag/Category System
**Description**: Add tags/categories to organize posts and improve discoverability.

**Implementation**:
- Add `Tag` entity with many-to-many relationship to `Post`
- Add tag input field to post creation/edit forms
- Add tag filtering to blog list view
- Add tag cloud widget

**Files to modify**:
- `migrations/` - New migration for tags table
- `models/src/` - New tag model
- `src/controllers/blog.rs` - Tag filtering
- `templates/blog/` - Tag UI components

**Benefits**: Better content organization, improved SEO, enhanced user experience

---

#### 2. Post Excerpt/Summary System
**Description**: Add excerpt field for posts to show previews in list view.

**Implementation**:
- Add `excerpt` column to `Post` model
- Modify create/edit forms to include excerpt input
- Update list template to show excerpts
- Auto-generate excerpts from content if not provided

**Files to modify**:
- `migrations/` - Add excerpt column
- `models/src/post.rs` - Add excerpt field
- `src/dto/post.rs` - Add excerpt to forms
- `templates/blog/list.html.tera` - Display excerpts

**Benefits**: Better content previews, improved user engagement

---

#### 3. Search Functionality
**Description**: Basic search across post titles and content.

**Implementation**:
- Add search endpoint to blog controller
- Create search service with PostgreSQL LIKE queries
- Add search form to header
- Create search results template

**Files to modify**:
- `src/controllers/blog.rs` - Search endpoint
- `src/services/blog.rs` - Search methods
- `templates/base.html.tera` - Search form
- New `templates/blog/search.html.tera`

**Benefits**: Improved content discoverability, better UX

---

#### 4. RSS/Atom Feed
**Description**: Generate RSS feed for published posts.

**Implementation**:
- Add RSS controller with XML response
- Create RSS template or use RSS library
- Add RSS link to site header
- Include post summaries and metadata

**Files to modify**:
- New `src/controllers/feed.rs`
- `src/main.rs` - Mount feed controller
- `templates/base.html.tera` - RSS link

**Benefits**: Better SEO, subscriber engagement, content syndication

---

### 🟡 Medium Priority - Valuable Additions (3-7 days implementation)

#### 5. Like/Reaction System
**Description**: Allow users to react to posts with likes/reactions.

**Implementation**:
- Add `PostReaction` entity linking posts to user sessions/IPs
- Add reaction buttons to post detail view
- Implement AJAX for real-time reaction updates
- Add reaction counts to post list

**Files to modify**:
- `migrations/` - New reactions table
- `models/src/` - New reaction model
- `src/controllers/blog.rs` - Reaction endpoints
- `templates/blog/detail.html.tera` - Reaction UI

**Benefits**: Increased engagement, user feedback, social proof

---

#### 6. Image Upload Support  
**Description**: Support image uploads in addition to videos.

**Implementation**:
- Extend file upload handling for images
- Add image resizing/optimization
- Support multiple file attachments per post
- Add image gallery display

**Files to modify**:
- `src/controllers/blog.rs` - Multiple file handling
- `src/services/blog.rs` - Image processing
- `models/src/post.rs` - Multiple file paths
- `templates/blog/` - Image display components

**Benefits**: Rich media content, better visual appeal

---

#### 7. Comment Moderation
**Description**: Admin tools for comment management.

**Implementation**:
- Add comment status field (pending/approved/spam)
- Create admin interface for comment management
- Add comment approval workflow
- Implement basic spam detection

**Files to modify**:
- `migrations/` - Add status to comments
- `models/src/comment.rs` - Status field
- New `src/controllers/admin.rs` - Admin interface
- New admin templates

**Benefits**: Content quality control, spam prevention

---

#### 8. User Profiles
**Description**: Extended user profiles with bio, avatar, social links.

**Implementation**:
- Extend Account model with profile fields
- Create user profile pages
- Add profile editing interface
- Link author names to profiles

**Files to modify**:
- `migrations/` - Extend account table
- `models/src/account.rs` - Profile fields
- `src/controllers/` - Profile controllers
- New profile templates

**Benefits**: Better author attribution, community building

---

### 🔴 Advanced Features - Complex Implementation (1-3 weeks)

#### 9. Full-Text Search with PostgreSQL
**Description**: Advanced search using PostgreSQL's full-text search capabilities.

**Implementation**:
- Set up PostgreSQL text search indexes
- Implement search ranking and highlighting
- Add search filters (date, author, tags)
- Create advanced search interface

**Benefits**: Powerful search capabilities, better performance

---

#### 10. Analytics Dashboard
**Description**: Admin dashboard showing site statistics and content performance.

**Implementation**:
- Track page views, popular posts, user engagement
- Create analytics data models
- Build dashboard with charts and metrics
- Add export functionality

**Benefits**: Data-driven content decisions, performance insights

---

#### 11. Multi-Author Support
**Description**: Support for multiple non-admin authors.

**Implementation**:
- Add author role system beyond admin/user
- Implement author permissions
- Add author assignment to posts
- Create author management interface

**Benefits**: Scalable content creation, team collaboration

---

#### 12. Content Scheduling
**Description**: Schedule posts for future publication.

**Implementation**:
- Add scheduled_date field to posts
- Create background job system for publishing
- Add scheduling interface to post forms
- Implement draft → scheduled → published workflow

**Benefits**: Content planning, automated publishing

---

#### 13. REST API
**Description**: RESTful API for blog operations.

**Implementation**:
- Add JSON endpoints for all operations
- Implement API authentication
- Add API documentation
- Support for mobile apps/third-party integration

**Benefits**: Platform extensibility, mobile app support

---

## Implementation Priority Recommendation

### Phase 1 (Week 1): Foundation
1. Tag/Category System
2. Post Excerpts
3. Basic Search

### Phase 2 (Week 2): Engagement
1. RSS Feed
2. Like/Reaction System
3. Image Upload Support

### Phase 3 (Week 3-4): Administration
1. Comment Moderation
2. User Profiles
3. Basic Analytics

### Phase 4 (Month 2+): Advanced
1. Full-Text Search
2. Multi-Author Support
3. Content Scheduling
4. REST API

---

## Technical Considerations

### Database Migration Strategy
- All new features should include proper migrations
- Maintain backward compatibility
- Use nullable fields for optional features

### Performance Considerations
- Add database indexes for search and filtering
- Implement caching for frequently accessed data
- Consider CDN for static assets

### Security Considerations
- Validate all user inputs
- Implement rate limiting for API endpoints
- Add CSRF protection for forms
- Sanitize uploaded content

### Testing Strategy
- Add unit tests for new services
- Integration tests for new endpoints
- UI tests for critical user flows

---

## Next Steps

1. **Stakeholder Review**: Review priorities with project stakeholders
2. **Technical Design**: Create detailed technical designs for selected features
3. **Development**: Implement features in priority order
4. **Testing**: Comprehensive testing of new functionality
5. **Documentation**: Update user and developer documentation

This roadmap provides a clear path for enhancing the Rocket Blog application while maintaining code quality and architectural integrity.