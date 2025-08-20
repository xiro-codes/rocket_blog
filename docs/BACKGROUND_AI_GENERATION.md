# Background AI Generation Feature

This feature enables asynchronous AI content generation to solve the problem of long wait times when generating blog content with AI services.

## Overview

Instead of waiting for AI generation to complete synchronously, users can now:
1. Submit an AI generation request and immediately receive a job ID
2. Continue using the application while AI processes in the background
3. Check job status and retrieve results when ready
4. Automatically get draft posts created when content generation completes

## API Endpoints

### Start Background AI Generation
**POST** `/blog/generate-content-async`

Starts AI generation in the background and returns immediately with a job ID.

**Request Body:**
```json
{
  "title": "My Blog Post Title",
  "prompt": "Additional context for AI generation (optional)",
  "type": "content|excerpt|tags",
  "provider": "openai|ollama (optional)"
}
```

**Response:**
```json
{
  "success": true,
  "job_id": "uuid-of-background-job",
  "status": "pending",
  "message": "AI generation started in background. Use /job-status/{job_id} to check progress."
}
```

### Check Job Status
**GET** `/blog/job-status/{job_id}`

Check the status of a background AI generation job.

**Response (Pending):**
```json
{
  "job_id": "uuid-of-background-job",
  "status": "pending",
  "created_at": "2024-12-06T12:00:00",
  "updated_at": "2024-12-06T12:00:00",
  "completed_at": null
}
```

**Response (Running):**
```json
{
  "job_id": "uuid-of-background-job",
  "status": "running",
  "created_at": "2024-12-06T12:00:00",
  "updated_at": "2024-12-06T12:00:30",
  "completed_at": null
}
```

**Response (Completed - Content Generation):**
```json
{
  "job_id": "uuid-of-background-job",
  "status": "completed",
  "created_at": "2024-12-06T12:00:00",
  "updated_at": "2024-12-06T12:02:15",
  "completed_at": "2024-12-06T12:02:15",
  "result": {
    "content": "Generated blog post content...",
    "provider": "OpenAI",
    "draft_post_id": "uuid-of-created-draft",
    "draft_post_title": "[AI DRAFT] My Blog Post Title"
  }
}
```

**Response (Failed):**
```json
{
  "job_id": "uuid-of-background-job",
  "status": "failed",
  "created_at": "2024-12-06T12:00:00",
  "updated_at": "2024-12-06T12:01:00",
  "completed_at": "2024-12-06T12:01:00",
  "error": "Error description"
}
```

## Automatic Draft Post Creation

When content generation completes successfully, the system automatically:

1. **Creates a draft post** with the generated content
2. **Adds "[AI DRAFT]" prefix** to the title for easy identification
3. **Sets draft status** to `true` so it's not published immediately
4. **Returns draft post information** in the job result

Users can then:
- Navigate to the blog edit page to review and refine the content
- Remove the "[AI DRAFT]" prefix from the title
- Publish the post when ready

## Job Processing

The background job processor:
- **Runs automatically** when the application starts
- **Processes jobs every 5 seconds** checking for pending work
- **Supports multiple job types**: content, excerpt, and tags generation
- **Uses existing AI providers** (OpenAI, Ollama) seamlessly
- **Handles failures gracefully** with error logging and job status updates

## Backwards Compatibility

The existing synchronous endpoint `/blog/generate-content` remains unchanged and fully functional. Users can choose between:
- **Synchronous generation**: Wait for immediate results (existing behavior)
- **Asynchronous generation**: Start job and poll for results (new feature)

## Usage Workflow

### Frontend Integration Example
```javascript
// Start background AI generation
const response = await fetch('/blog/generate-content-async', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    title: 'My Blog Post',
    prompt: 'Write about web development',
    type: 'content'
  })
});

const { job_id } = await response.json();

// Poll for completion
const pollJob = async () => {
  const statusResponse = await fetch(`/blog/job-status/${job_id}`);
  const status = await statusResponse.json();
  
  if (status.status === 'completed') {
    console.log('Draft post created:', status.result.draft_post_id);
    // Redirect to edit the draft post
    window.location.href = `/blog/edit/${status.result.draft_post_id}`;
  } else if (status.status === 'failed') {
    console.error('Generation failed:', status.error);
  } else {
    // Still processing, check again in 3 seconds
    setTimeout(pollJob, 3000);
  }
};

// Start polling
setTimeout(pollJob, 3000);
```

## Database Schema

The feature adds a new `background_job` table:

```sql
CREATE TABLE background_job (
  id UUID PRIMARY KEY,
  job_type VARCHAR NOT NULL,
  status VARCHAR NOT NULL DEFAULT 'pending',
  payload JSON,
  result JSON,
  error TEXT,
  created_at TIMESTAMP NOT NULL,
  updated_at TIMESTAMP NOT NULL,
  completed_at TIMESTAMP,
  account_id UUID NOT NULL REFERENCES account(id)
);
```

## Benefits

✅ **No more waiting**: Users get immediate response and can continue working  
✅ **Automatic drafts**: Generated content becomes editable draft posts  
✅ **Reliable processing**: Jobs are persisted and processed reliably  
✅ **Error handling**: Failed jobs are tracked with error details  
✅ **Scalable**: Background processing can handle multiple concurrent requests  
✅ **Compatible**: Existing synchronous API remains unchanged  

This solves the original problem of "AI features take too long" while providing a superior user experience with automatic draft creation.