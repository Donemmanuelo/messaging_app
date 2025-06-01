# Development Guide

## Development Workflow

### Git Workflow

1. **Branching Strategy**
   ```bash
   main          # Production branch
   ├── develop   # Development branch
   ├── feature/* # Feature branches
   ├── bugfix/*  # Bug fix branches
   └── release/* # Release branches
   ```

2. **Branch Naming Convention**
   - Feature: `feature/JIRA-123-short-description`
   - Bugfix: `bugfix/JIRA-123-short-description`
   - Release: `release/v1.2.3`

3. **Commit Message Format**
   ```
   <type>(<scope>): <subject>

   <body>

   <footer>
   ```
   Types: feat, fix, docs, style, refactor, test, chore

### Code Review Process

1. **Pull Request Template**
   ```markdown
   ## Description
   [Detailed description of changes]

   ## Type of Change
   - [ ] Bug fix
   - [ ] New feature
   - [ ] Breaking change
   - [ ] Documentation update

   ## Testing
   - [ ] Unit tests added/updated
   - [ ] Integration tests added/updated
   - [ ] Manual testing completed

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Documentation updated
   - [ ] Tests added/updated
   - [ ] All tests passing
   ```

2. **Review Guidelines**
   - Code style and consistency
   - Test coverage
   - Documentation updates
   - Performance considerations
   - Security implications

### Release Process

1. **Version Management**
   - Semantic versioning (MAJOR.MINOR.PATCH)
   - Changelog maintenance
   - Release notes generation

2. **Release Steps**
   ```bash
   # Create release branch
   git checkout -b release/v1.2.3 develop

   # Update version
   cargo set-version 1.2.3

   # Create changelog
   git-chglog -o CHANGELOG.md

   # Merge to main and develop
   git checkout main
   git merge release/v1.2.3
   git tag v1.2.3
   git checkout develop
   git merge release/v1.2.3
   ```

## Development Environment

### Prerequisites
- Rust (latest stable)
- Docker and Docker Compose
- Node.js 16+ (for frontend)
- PostgreSQL 14+
- Redis 6+
- IDE (VS Code recommended)

### Environment Setup

1. **Clone Repository**
   ```bash
   git clone https://github.com/your-org/messaging-app.git
   cd messaging-app
   ```

2. **Environment Variables**
   ```env
   # Database
   DB_HOST=localhost
   DB_PORT=5432
   DB_NAME=messaging_app
   DB_USER=postgres
   DB_PASSWORD=your_password

   # Redis
   REDIS_HOST=localhost
   REDIS_PORT=6379

   # JWT
   JWT_SECRET=your_secret
   JWT_EXPIRATION=86400

   # Media Storage
   S3_BUCKET=your-bucket
   S3_REGION=your-region
   S3_ACCESS_KEY=your-key
   S3_SECRET_KEY=your-secret
   ```

3. **Development Tools**
   ```bash
   # Install development tools
   cargo install cargo-watch
   cargo install cargo-tarpaulin
   cargo install cargo-audit
   cargo install cargo-flamegraph
   ```

## Testing Strategy

### Unit Testing

1. **Test Organization**
   ```
   tests/
   ├── unit/
   │   ├── models/
   │   ├── services/
   │   └── utils/
   ├── integration/
   └── e2e/
   ```

2. **Test Coverage Requirements**
   - Minimum 80% code coverage
   - Critical paths: 100% coverage
   - Edge cases covered
   - Error scenarios tested

### Integration Testing

1. **Test Environment**
   ```bash
   # Start test environment
   docker-compose -f docker-compose.test.yml up -d

   # Run tests
   cargo test -- --test-threads=1
   ```

2. **API Testing**
   ```rust
   #[tokio::test]
   async fn test_message_flow() {
       let app = create_test_app().await;
       let client = TestClient::new(app);

       // Test message creation
       let response = client
           .post("/api/messages")
           .json(&json!({
               "receiver_id": "user_123",
               "content": "Hello!"
           }))
           .send()
           .await;

       assert_eq!(response.status(), StatusCode::OK);
   }
   ```

### Performance Testing

1. **Load Testing**
   ```javascript
   import http from 'k6/http';
   import { check, sleep } from 'k6';

   export const options = {
       stages: [
           { duration: '30s', target: 20 },
           { duration: '1m', target: 50 },
           { duration: '30s', target: 0 },
       ],
   };

   export default function() {
       const response = http.post('http://localhost:8080/api/messages', {
           receiver_id: 'user_123',
           content: 'Hello!',
       });

       check(response, {
           'status is 200': (r) => r.status === 200,
           'response time < 200ms': (r) => r.timings.duration < 200,
       });

       sleep(1);
   }
   ```

2. **Benchmarking**
   ```bash
   # Run benchmarks
   cargo bench

   # Profile performance
   cargo flamegraph
   ```

## Code Quality

### Static Analysis

1. **Rust Code Style**
   ```bash
   # Format code
   cargo fmt

   # Lint code
   cargo clippy

   # Check for security issues
   cargo audit
   ```

2. **TypeScript/JavaScript**
   ```bash
   # Lint code
   npm run lint

   # Type check
   npm run type-check
   ```

### Documentation

1. **Code Documentation**
   ```rust
   /// Represents a chat message
   #[derive(Debug, Serialize, Deserialize)]
   pub struct Message {
       /// Unique identifier
       pub id: Uuid,
       /// Message content
       pub content: String,
       /// Sender's ID
       pub sender_id: Uuid,
       /// Receiver's ID
       pub receiver_id: Uuid,
       /// Creation timestamp
       pub created_at: DateTime<Utc>,
   }
   ```

2. **API Documentation**
   ```rust
   /// Send a new message
   ///
   /// # Arguments
   ///
   /// * `receiver_id` - The ID of the message recipient
   /// * `content` - The message content
   ///
   /// # Returns
   ///
   /// Returns the created message
   ///
   /// # Errors
   ///
   /// Returns an error if:
   /// * The receiver doesn't exist
   /// * The user is not authorized
   /// * The message content is invalid
   #[post("/messages")]
   pub async fn send_message(
       // ... implementation
   ) -> Result<Message, AppError> {
       // ... implementation
   }
   ```

## Security

### Security Scanning

1. **Dependency Scanning**
   ```bash
   # Scan dependencies
   cargo audit
   npm audit
   ```

2. **Code Scanning**
   ```bash
   # Run security scanner
   cargo-deny check
   ```

### Security Best Practices

1. **Input Validation**
   ```rust
   pub fn validate_message(content: &str) -> Result<(), ValidationError> {
       if content.is_empty() {
           return Err(ValidationError::EmptyContent);
       }
       if content.len() > MAX_MESSAGE_LENGTH {
           return Err(ValidationError::TooLong);
       }
       Ok(())
   }
   ```

2. **Error Handling**
   ```rust
   pub async fn handle_error(error: AppError) -> impl IntoResponse {
       match error {
           AppError::Validation(e) => (StatusCode::BAD_REQUEST, e.to_string()),
           AppError::Authentication(e) => (StatusCode::UNAUTHORIZED, e.to_string()),
           AppError::Authorization(e) => (StatusCode::FORBIDDEN, e.to_string()),
           _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
       }
   }
   ```

## Performance

### Optimization Guidelines

1. **Database Optimization**
   ```sql
   -- Add indexes
   CREATE INDEX idx_messages_sender_receiver ON messages(sender_id, receiver_id);
   CREATE INDEX idx_messages_created_at ON messages(created_at);

   -- Optimize queries
   EXPLAIN ANALYZE SELECT * FROM messages 
   WHERE sender_id = $1 AND receiver_id = $2 
   ORDER BY created_at DESC 
   LIMIT 50;
   ```

2. **Caching Strategy**
   ```rust
   // Cache frequently accessed data
   pub async fn get_user_profile(user_id: Uuid) -> Result<UserProfile, AppError> {
       let cache_key = format!("user:{}", user_id);
       
       // Try cache first
       if let Some(profile) = redis.get(&cache_key).await? {
           return Ok(profile);
       }

       // Get from database
       let profile = db.get_user_profile(user_id).await?;
       
       // Cache for 1 hour
       redis.set_ex(&cache_key, &profile, 3600).await?;
       
       Ok(profile)
   }
   ```

## Maintenance

### Dependency Management

1. **Update Dependencies**
   ```bash
   # Update Rust dependencies
   cargo update

   # Update npm dependencies
   npm update
   ```

2. **Check for Updates**
   ```bash
   # Check for outdated dependencies
   cargo outdated
   npm outdated
   ```

### Code Maintenance

1. **Refactoring Guidelines**
   - Keep functions small and focused
   - Use meaningful names
   - Remove dead code
   - Update documentation

2. **Technical Debt**
   - Track debt items
   - Regular reviews
   - Prioritize fixes
   - Document decisions 