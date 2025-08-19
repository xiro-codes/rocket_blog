# AI Integration - OpenAI and Ollama Support

This document describes how to set up and use both OpenAI and Ollama for automated blog post generation.

## Overview

The blog application now supports two AI providers:

1. **OpenAI** - Cloud-based AI service (GPT-3.5-turbo)
2. **Ollama** - Local/self-hosted AI models

Both providers offer the same functionality:
- Content generation from titles and prompts
- Excerpt generation from existing content  
- Tag generation based on content and titles

## OpenAI Setup

### 1. Get OpenAI API Key

1. Sign up for an OpenAI account at https://platform.openai.com/
2. Navigate to API Keys section
3. Create a new API key

### 2. Configure OpenAI API Key

**Database Configuration (Recommended)**

1. **Access Settings**: Log in as an admin and navigate to Settings from the admin panel
2. **Configure API Key**: Enter your OpenAI API key in the settings form  
3. **Validation**: The system will test the API key before saving
4. **Enable Features**: AI generation buttons will appear in the blog editor once configured

## Ollama Setup

### 1. Install Ollama

Visit https://ollama.ai/ and follow the installation instructions for your platform.

For Linux/macOS:
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

### 2. Install a Model

```bash
# Install a model (e.g., llama2)
ollama pull llama2

# Or install a smaller model for testing
ollama pull llama2:7b
```

### 3. Start Ollama Service

```bash
# Start Ollama server (default: http://localhost:11434)
ollama serve
```

### 4. Configure Ollama in Blog Settings

1. **Access Settings**: Log in as an admin and navigate to Settings
2. **Ollama URL**: Enter your Ollama server URL (default: `http://localhost:11434`)
3. **Model**: Specify which model to use (default: `llama2`)
4. **Enable**: Toggle Ollama support on/off

## Usage

### AI Provider Selection

The system automatically selects the first available AI provider. Provider availability is checked in this order:

1. OpenAI (if API key is configured and valid)
2. Ollama (if URL is configured and server is reachable)

### API Endpoint

Send POST requests to `/blog/generate-content` with the following parameters:

```json
{
  "type": "content|excerpt|tags",
  "title": "Your blog post title",
  "content": "Content for excerpt/tag generation",
  "prompt": "Additional context (optional)",
  "provider": "openai|ollama (optional)"
}
```

#### Provider Selection

- If `provider` is specified, the system will use that specific provider
- If `provider` is not specified, the system uses the first available provider
- The response includes which provider was actually used

#### Response Format

```json
{
  "success": true,
  "content": "Generated content...",
  "provider": "OpenAI"
}
```

## Feature Comparison

| Feature | OpenAI | Ollama |
|---------|--------|--------|
| Cloud-based | ✅ | ❌ |
| Self-hosted | ❌ | ✅ |
| API Cost | Per token | Free (after setup) |
| Internet Required | ✅ | ❌ |
| Model Selection | Fixed (GPT-3.5-turbo) | Flexible |
| Setup Complexity | Easy | Moderate |

## Configuration Examples

### OpenAI Settings
- **API Key**: `sk-...` (encrypted in database)
- **Model**: `gpt-3.5-turbo` (fixed)

### Ollama Settings
- **URL**: `http://localhost:11434`
- **Model**: `llama2` or `mistral`, `codellama`, etc.
- **Enabled**: `true`/`false`

## Troubleshooting

### OpenAI Issues

1. **"OpenAI service not configured"**
   - Check that API key is set in admin settings
   - Verify API key is valid

2. **"OpenAI API error"**
   - Check API key permissions
   - Verify sufficient credits
   - Check network connectivity

### Ollama Issues

1. **"No AI service configured"**
   - Check that Ollama URL is set in admin settings
   - Verify Ollama server is running
   - Test connection: `curl http://localhost:11434/api/tags`

2. **"Ollama connection test failed"**
   - Ensure Ollama service is running: `ollama serve`
   - Check URL is correct
   - Verify firewall settings

3. **Model not found errors**
   - Install the model: `ollama pull <model-name>`
   - Check available models: `ollama list`

### General Issues

1. **Empty responses**
   - Try rephrasing the title/prompt
   - Check if content is too short for processing

2. **Slow responses**
   - OpenAI: Check internet connection
   - Ollama: Consider using smaller models or better hardware

## Security

### OpenAI
- API keys are encrypted using AES-256-GCM before database storage
- No API key exposure in client-side code
- Admin-only access to settings configuration

### Ollama  
- URL configuration stored in plain text (typically localhost)
- No authentication required for local Ollama instances
- Consider network security for remote Ollama instances

## Performance Considerations

### OpenAI
- Cloud-based, response time depends on internet connection
- Rate limited by OpenAI (varies by plan)
- Estimated costs:
  - Content generation: ~$0.003 per post
  - Excerpt generation: ~$0.0003 per post
  - Tag generation: ~$0.0002 per post

### Ollama
- Local processing, response time depends on hardware
- No external rate limits
- CPU/Memory intensive for larger models
- Consider GPU acceleration for better performance

## Model Recommendations

### Ollama Models for Different Use Cases

- **Small/Fast**: `llama2:7b`, `mistral:7b`
- **Balanced**: `llama2`, `mistral`
- **Large/Quality**: `llama2:70b`, `mixtral:8x7b`
- **Code-focused**: `codellama`, `deepseek-coder`

Choose based on your hardware capabilities and quality requirements.