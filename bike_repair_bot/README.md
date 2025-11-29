# 🏍️ Bike Repair ChatBot - Rust Backend

An AI-powered motorcycle repair and maintenance chatbot with embedded RAG pipeline and vector database.

## Features

- ✅ **Monolithic Rust Backend** - Single binary with all components
- ✅ **OpenAI Integration** - GPT-4o-mini for chat, text-embedding-3-small for embeddings
- ✅ **Enterprise Security** - Rate limiting, query validation, circuit breaker
- ✅ **RESTful API** - Warp-based HTTP server with CORS support
- 🚧 **Embedded RAG Pipeline** - (Coming in Phase 5)
- 🚧 **PDF Processing** - (Coming in Phase 6)
- 🚧 **Flutter Mobile App** - (Coming in Phase 9)

## Current Status

**Phase 3 Complete**: Core infrastructure implemented
- ✅ Configuration management
- ✅ Data models
- ✅ Security layer (rate limiting, validation, circuit breaker)
- ✅ OpenAI client
- ✅ HTTP server with endpoints
- ✅ Error handling

## Setup

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs/)
- OpenAI API key

### Installation

1. Clone the repository
```bash
cd d:\upwork\Upwork-clinet\bike_repair_bot
```

2. Create `.env` file from template
```bash
copy .env.example .env
```

3. Edit `.env` and add your OpenAI API key:
```env
OPENAI_API_KEY=sk-your-actual-api-key-here
```

4. Build the project
```bash
cargo build --release
```

5. Run the server
```bash
cargo run --release
```

## API Endpoints

### Health Check
```bash
GET /api/health
```

Response:
```json
{
  "status": "healthy",
  "service": "Bike Repair ChatBot",
  "version": "0.1.0"
}
```

### Chat
```bash
POST /api/chat
Content-Type: application/json

{
  "query": "How do I change motorcycle oil?",
  "session_id": "optional-session-id",
  "bike_model": "Honda CBR600RR"
}
```

Response:
```json
{
  "response": "To change motorcycle oil...",
  "session_id": "uuid",
  "sources": [],
  "rate_limit_info": {
    "remaining_minute": 19,
    "remaining_hour": 99,
    "reset_in_seconds": 0
  }
}
```

### Status
```bash
GET /api/status
```

Response:
```json
{
  "rate_limit": {
    "remaining_minute": 20,
    "remaining_hour": 100,
    "reset_in_seconds": 0
  },
  "circuit_breaker": {
    "state": "Closed"
  }
}
```

## Testing

### Using curl

```bash
# Health check
curl http://localhost:8080/api/health

# Chat request
curl -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"How do I change oil on Honda CBR600RR?\"}"

# Check status
curl http://localhost:8080/api/status
```

### Using Postman

Import the following environment:
- URL: `http://localhost:8080`

Try the `/api/chat` endpoint with different queries:
- ✅ "How to adjust motorcycle chain?" (valid)
- ✅ "Honda CBR brake maintenance" (valid)
- ❌ "What's the weather?" (rejected - not bike-related)

## Configuration

All configuration is in `.env`:

| Variable | Default | Description |
|----------|---------|-------------|
| `OPENAI_API_KEY` | - | Required: Your OpenAI API key |
| `SERVER_PORT` | 8080 | HTTP server port |
| `SERVER_HOST` | 0.0.0.0 | Server bind address |
| `RUST_LOG` | info | Logging level |
| `MAX_REQUESTS_PER_MINUTE` | 20 | Rate limit per minute |
| `MAX_REQUESTS_PER_HOUR` | 100 | Rate limit per hour |
| `CIRCUIT_BREAKER_THRESHOLD` | 5 | Failures before circuit opens |
| `OPENAI_CHAT_MODEL` | gpt-4o-mini | Chat model to use |
| `OPENAI_EMBEDDING_MODEL` | text-embedding-3-small | Embedding model |

## Project Structure

```
bike_repair_bot/
├── src/
│   ├── main.rs                 # Application entry point
│   ├── config.rs               # Configuration management
│   ├── models/                 # Data models
│   │   ├── chat.rs            # Chat request/response
│   │   └── document.rs        # Document/chunk models
│   ├── security/              # Security layer
│   │   ├── rate_limiter.rs   # Per-IP rate limiting
│   │   ├── validator.rs      # Query validation
│   │   └── circuit_breaker.rs # Circuit breaker pattern
│   ├── ai/                    # OpenAI integration
│   │   ├── openai_client.rs  # API client
│   │   └── prompts.rs        # Prompt engineering
│   ├── server/                # HTTP server
│   │   ├── routes.rs         # Route definitions
│   │   └── handlers.rs       # Request handlers
│   ├── rag/                   # RAG pipeline (placeholder)
│   └── pdf/                   # PDF processing (placeholder)
├── Cargo.toml                  # Dependencies
├── .env                        # Environment variables
└── README.md                   # This file
```

## Next Steps

- [ ] **Phase 5**: Implement embedded RAG pipeline with Qdrant
- [ ] **Phase 6**: Add PDF upload and processing
- [ ] **Phase 7**: Enhanced prompt engineering
- [ ] **Phase 9**: Build Flutter mobile app
- [ ] **Phase 10**: Integration testing
- [ ] **Phase 11**: Deployment preparation

## Security Features

1. **Rate Limiting**: Per-IP limits to prevent abuse
   - 20 requests per minute
   - 100 requests per hour
   - Automatic cooldown periods

2. **Query Validation**: Ensures bike-related queries only
   - Keyword matching
   - SQL injection prevention
   - XSS protection
   - Special character filtering

3. **Circuit Breaker**: Protects against API failures
   - Opens after 5 consecutive failures
   - Half-open recovery testing
   - Automatic closure on success

## Troubleshooting

### "OPENAI_API_KEY must be set"
Make sure you've created a `.env` file with your API key.

### Port already in use
Change `SERVER_PORT` in `.env` to a different port.

### Rate limit exceeded
Wait for the cooldown period indicated in the error message.

## License

Proprietary - Upwork Client Project

## Author

Built for Upwork client project
