# OpenAI Integration

This document describes how to set up and use the OpenAI integration for automated blog post generation.

## Setup

### 1. Get OpenAI API Key

1. Sign up for an OpenAI account at https://platform.openai.com/
2. Navigate to API Keys section
3. Create a new API key

### 2. Configure API Key

Add your OpenAI API key to your environment:

```bash
# In .env file
OPENAI_API_KEY=your-openai-api-key-here
```

Or set as environment variable:

```bash
export OPENAI_API_KEY=your-openai-api-key-here
```

## Features

The OpenAI integration provides three main features:

### 1. Content Generation
- Generates full blog post content from a title
- Supports additional prompts for context
- Uses GPT-3.5-turbo model with optimized parameters

### 2. Excerpt Generation
- Creates engaging 1-2 sentence summaries
- Automatically extracts key points from content
- Perfect for blog post listings

### 3. Tag Generation
- Suggests relevant tags based on title and content
- Returns comma-separated list of tags
- Helps with content categorization

## Usage

### Web Interface

When creating a new blog post:

1. **Generate Content**: 
   - Enter a title
   - Optionally add a prompt for additional context
   - Click "✨ Generate" to create full content

2. **Generate Excerpt**:
   - Write or generate content first
   - Click "✨ Generate Excerpt" to create summary

3. **Generate Tags**:
   - Enter title and/or content
   - Click "✨ Generate Tags" to get suggestions

### API Endpoint

POST `/blog/generate-content`

```json
{
  "type": "content|excerpt|tags",
  "title": "Your Post Title",
  "content": "Post content (for excerpt/tags)",
  "prompt": "Additional context (optional)"
}
```

**Response:**
```json
{
  "success": true,
  "content": "Generated content...",
  "excerpt": "Generated excerpt...",
  "tags": "tag1, tag2, tag3"
}
```

## Cost Considerations

- GPT-3.5-turbo is used for cost efficiency
- Content generation: ~1500 tokens max
- Excerpt generation: ~150 tokens max  
- Tag generation: ~100 tokens max

Estimated costs (as of 2024):
- Content generation: ~$0.003 per post
- Excerpt generation: ~$0.0003 per post
- Tag generation: ~$0.0002 per post

## Error Handling

The system gracefully handles:
- Missing API key (shows configuration message)
- API rate limits (shows error to user)
- Network issues (shows retry message)
- Invalid responses (falls back to manual input)

## Security

- API key is stored securely in environment variables
- Requests are authenticated (admin only)
- No API key exposure in client-side code
- Rate limiting handled by OpenAI

## Troubleshooting

### Common Issues

1. **"OpenAI service not configured"**
   - Check that OPENAI_API_KEY is set
   - Verify API key is valid

2. **"OpenAI API error: ..."**
   - Check API key permissions
   - Verify sufficient credits
   - Check network connectivity

3. **Empty responses**
   - Try rephrasing the title/prompt
   - Check if content is too short for excerpt generation

### Debug Mode

Set environment variable for debugging:
```bash
export RUST_LOG=debug
```

This will log OpenAI API requests and responses.