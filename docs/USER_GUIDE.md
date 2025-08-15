# User Guide

Welcome to Rocket Blog! This guide will help you understand how to use all the features of your new blog platform.

## 🚀 Getting Started

### First Time Setup

1. **Access Your Blog**
   - Open your web browser and navigate to your blog URL
   - For local development: `http://localhost:8000/blog`

2. **Admin Login**
   - Click the login link or navigate to `/auth`
   - Use the admin credentials created during setup
   - You'll be redirected to the blog dashboard after successful login

3. **Create Your First Post**
   - Click "New Post" button (visible when logged in as admin)
   - Fill in the title and content
   - Add tags to organize your content
   - Click "Publish" or "Save as Draft"

## 📝 Writing Blog Posts

### Creating a New Post

1. **Navigate to Blog Creation**
   - Go to `/blog/create` or click "New Post"
   - You must be logged in as an admin

2. **Fill in Post Details**
   ```
   Title: Your Post Title
   Content: Your post content in Markdown format
   Tags: tag1, tag2, tag3 (comma-separated)
   ```

3. **Using Markdown**
   Your blog supports full Markdown syntax:

   ```markdown
   # Heading 1
   ## Heading 2
   ### Heading 3
   
   **Bold text**
   *Italic text*
   
   [Link text](https://example.com)
   
   ![Image description](image-url)
   
   ```
   Code blocks with syntax highlighting
   ```
   
   - Bullet list item 1
   - Bullet list item 2
   
   1. Numbered list item 1
   2. Numbered list item 2
   
   > Blockquote text
   
   | Table | Headers |
   |-------|---------|
   | Row 1 | Data    |
   | Row 2 | Data    |
   ```

4. **Adding Tags**
   - Tags help organize your content
   - Separate multiple tags with commas
   - Tags are automatically created if they don't exist
   - Examples: `rust, web development, tutorial, programming`

5. **Publishing**
   - **Publish**: Makes the post immediately visible to readers
   - **Save as Draft**: Saves the post but keeps it hidden from readers

### Editing Existing Posts

1. **Find Your Post**
   - Navigate to the blog list at `/blog`
   - Click on the post title to view it

2. **Edit Mode**
   - Click the "Edit" button (visible when logged in as admin)
   - Modify title, content, or tags as needed
   - Click "Update" to save changes

3. **Managing Published Status**
   - You can change between published and draft status when editing
   - Draft posts are only visible to admin users

### Deleting Posts

1. **Navigate to Post**
   - Go to the specific post you want to delete

2. **Delete Action**
   - Click the "Delete" button (visible when logged in as admin)
   - Confirm the deletion
   - **Warning**: This action cannot be undone

## 🏷️ Using Tags

### What Are Tags?

Tags are labels that help organize and categorize your blog posts. They make it easier for readers to find related content.

### Adding Tags to Posts

1. **During Creation**
   - Add tags in the "Tags" field when creating a post
   - Use comma-separated values: `web, javascript, tutorial`

2. **When Editing**
   - Edit existing posts to add or remove tags
   - Tags are automatically saved with the post

### Tag Features

- **Automatic Creation**: New tags are created automatically when used
- **Color Coding**: Each tag gets a unique color for visual distinction
- **Filtering**: Readers can click tags to see related posts (UI pending)
- **Management**: Admin users can manage tags through the backend

## 💬 Comments System

### How Comments Work

- **Reader Comments**: Any visitor can leave comments on published posts
- **No Registration Required**: Comments don't require user accounts
- **Moderation**: Admin users can moderate comments

### Managing Comments

1. **Viewing Comments**
   - Comments appear at the bottom of each blog post
   - Admin users see additional management options

2. **Comment Moderation** (Admin Feature)
   - Review comments before they're published
   - Delete inappropriate or spam comments
   - Respond to reader questions and feedback

### Best Practices for Comments

- **Encourage Engagement**: Ask questions at the end of posts
- **Respond Promptly**: Reply to comments to build community
- **Moderate Fairly**: Remove spam but preserve constructive criticism
- **Set Guidelines**: Consider adding comment guidelines to your blog

## 🔐 Authentication and Security

### Admin Access

1. **Logging In**
   - Navigate to `/auth`
   - Enter your username and password
   - You'll be redirected to the blog after successful login

2. **Admin Features**
   When logged in as admin, you can:
   - Create new blog posts
   - Edit existing posts
   - Delete posts
   - Moderate comments
   - Access admin-only features

3. **Logging Out**
   - Click "Logout" when logged in
   - Or navigate to `/auth/logout`
   - Your session will be securely terminated

### Security Best Practices

- **Strong Passwords**: Use complex passwords for admin accounts
- **Regular Logouts**: Log out when using shared computers
- **HTTPS**: Always use HTTPS in production environments
- **Regular Updates**: Keep your blog software updated

## 📱 User Interface

### Navigation

- **Home Page**: Main blog listing with recent posts
- **Individual Posts**: Click any post title to read the full content
- **Pagination**: Use page navigation for older posts
- **Admin Controls**: Visible only when logged in as admin

### Responsive Design

Your blog is fully responsive and works on:
- **Desktop Computers**: Full-featured experience
- **Tablets**: Optimized layout for medium screens
- **Mobile Phones**: Touch-friendly mobile interface

### Dark Theme

The blog uses a modern dark theme by default:
- **Easy on Eyes**: Reduced eye strain for reading
- **Modern Aesthetic**: Clean, professional appearance
- **Bootstrap-based**: Consistent, well-tested UI components

## 🎬 Media Features

### Video Streaming

Your blog supports video file uploads and streaming:

1. **Uploading Videos**
   - Videos can be included in blog posts
   - Supports large files up to 1GB
   - Automatic streaming optimization

2. **Viewing Videos**
   - Videos stream efficiently with range request support
   - Viewers can seek to any position without full download
   - Works across all modern browsers

### File Uploads

- **Size Limit**: Files up to 1GB are supported
- **Security**: Uploaded files are safely stored
- **Access Control**: File access respects post privacy settings

## 📊 Blog Analytics (Coming Soon)

Future versions will include:
- **View Counts**: Track how many people read each post
- **Popular Posts**: See which content performs best
- **Reader Engagement**: Monitor comments and interactions
- **Search Insights**: Understand what readers are looking for

## 🔍 Search Features (Coming Soon)

Planned search capabilities:
- **Full-Text Search**: Search across all post content
- **Tag Filtering**: Filter posts by specific tags
- **Date Filtering**: Find posts from specific time periods
- **Advanced Search**: Complex search queries with operators

## 📧 RSS Feeds (Coming Soon)

Future RSS feed features:
- **Full Blog Feed**: Subscribe to all new posts
- **Tag-Specific Feeds**: Subscribe to posts with specific tags
- **Comment Feeds**: Subscribe to new comments
- **Podcast Support**: RSS feeds for audio content

## 🎨 Customization

### Current Customization Options

- **Content Organization**: Use tags to organize your posts
- **Post Formatting**: Rich Markdown support for content formatting
- **Media Integration**: Include images and videos in posts

### Future Customization Features

- **Theme Selection**: Choose from multiple blog themes
- **Custom CSS**: Add custom styling to your blog
- **Layout Options**: Different page layouts and structures
- **Widget System**: Add custom widgets to your blog

## 🛠️ Troubleshooting

### Common Issues

1. **Can't Log In**
   - Check username and password spelling
   - Ensure caps lock is not enabled
   - Clear browser cookies and try again
   - Contact your system administrator

2. **Post Not Saving**
   - Check your internet connection
   - Ensure you're still logged in
   - Try refreshing the page and logging in again
   - Check file size limits for large uploads

3. **Images Not Displaying**
   - Verify image URLs are correct
   - Ensure images are publicly accessible
   - Check internet connection
   - Try different image formats (JPG, PNG, GIF)

4. **Comments Not Appearing**
   - Comments may require moderation approval
   - Check if comments are enabled for the post
   - Ensure JavaScript is enabled in your browser
   - Try refreshing the page

### Getting Help

- **Documentation**: Check the full documentation in the `/docs` folder
- **GitHub Issues**: Report bugs at the project GitHub repository
- **Community**: Join community discussions for help and tips

## 📚 Best Practices

### Writing Great Blog Posts

1. **Compelling Titles**
   - Use clear, descriptive titles
   - Include keywords readers might search for
   - Keep titles under 60 characters for best display

2. **Quality Content**
   - Write for your audience
   - Use clear, concise language
   - Include examples and practical information
   - Proofread before publishing

3. **SEO-Friendly Posts**
   - Use descriptive headings (H1, H2, H3)
   - Include relevant tags
   - Add alt text to images
   - Link to related posts

4. **Engagement**
   - End posts with questions to encourage comments
   - Respond to reader comments promptly
   - Share posts on social media
   - Cross-reference related posts

### Content Organization

1. **Tag Strategy**
   - Use consistent tag naming
   - Don't over-tag posts (3-5 tags is usually enough)
   - Create tag hierarchies for better organization
   - Review and consolidate similar tags periodically

2. **Content Planning**
   - Plan content themes and series
   - Maintain a consistent publishing schedule
   - Balance different types of content
   - Keep a list of post ideas for future reference

### Maintenance Tasks

1. **Regular Updates**
   - Review and update older posts periodically
   - Fix broken links and images
   - Update outdated information
   - Improve SEO of popular posts

2. **Comment Management**
   - Check for new comments regularly
   - Respond to reader questions
   - Remove spam or inappropriate comments
   - Encourage community discussion

3. **Performance Monitoring**
   - Monitor blog loading speed
   - Check for broken links
   - Review popular content
   - Analyze reader engagement

---

**Congratulations!** You're now ready to create amazing content with Rocket Blog. Happy blogging! 🎉

For more advanced features and technical details, see the [full documentation](docs/) or check out the [API documentation](docs/API.md).