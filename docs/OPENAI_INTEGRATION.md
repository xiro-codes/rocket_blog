# OpenAI Integration - Admin Features

This document describes the new OpenAI-powered post generation feature and admin settings interface.

## Features Added

### 1. Admin Settings Interface (`/admin/settings`)
- Configure OpenAI API settings
- Set API key, model, max tokens, and temperature
- Secure password field for API key
- Model selection (GPT-3.5 Turbo, GPT-4, GPT-4 Turbo)

### 2. AI Post Generation (`/admin/generate`)
- Generate blog posts using OpenAI
- Provide a topic and get a full blog post
- Generated posts include title, content, and excerpt
- Content is pre-filled in the blog creation form

### 3. Settings Management
- Database-backed configuration storage
- Update settings without code changes
- Default values for all configuration options

## Setup Instructions

### 1. Environment Configuration
Add your OpenAI API key to the `.env` file:
```bash
OPENAI_API_KEY=your-openai-api-key-here
OPENAI_MODEL=gpt-3.5-turbo
```

### 2. Database Migration
The settings table is automatically created when the application starts. The migration includes default values for all OpenAI settings.

### 3. Admin Access
- Only admin users can access `/admin/settings` and `/admin/generate`
- Admin authentication is required for all admin endpoints
- Links are visible in the main blog interface when logged in as admin

## Usage Flow

1. **Configure Settings**: Go to `/admin/settings` to set up your OpenAI API key and preferences
2. **Generate Post**: Use `/admin/generate` to create AI-powered blog posts
3. **Review & Publish**: Generated content is pre-filled in the blog creation form for review and editing
4. **Publish**: Use the normal blog publishing workflow

## Navigation

Admin features are accessible from the main blog page when logged in:
- `[AI GENERATE]` button - Quick access to post generation
- `[ADMIN SETTINGS]` button - Configure OpenAI settings

## Technical Implementation

### Database Schema
New `settings` table stores configuration:
- `id` (UUID, Primary Key)
- `key` (String, Unique)
- `value` (Optional String)
- `description` (Optional String)
- `created_at` / `updated_at` (Timestamps)

### Services
- `SettingsService` - Manages configuration storage
- `OpenAIService` - Handles API communication with OpenAI
- `CoordinatorService` - Updated to include new services

### Controllers
- `AdminController` - New controller for admin-only features
- Routes: `/admin/settings` (GET/POST), `/admin/generate` (GET/POST)

### Security
- All admin endpoints require authentication
- API keys are stored securely in the database
- Input validation on all forms

## API Configuration Options

| Setting | Description | Default Value |
|---------|-------------|---------------|
| `openai_api_key` | Your OpenAI API key | (empty) |
| `openai_model` | Model to use | `gpt-3.5-turbo` |
| `openai_max_tokens` | Maximum response tokens | `1000` |
| `openai_temperature` | Creativity level (0.0-1.0) | `0.7` |

## Error Handling

The system gracefully handles:
- Missing or invalid API keys
- OpenAI API errors
- Network connectivity issues
- Invalid configuration values

Error messages are displayed to the user with helpful context.