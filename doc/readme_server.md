# Wind-KVStore Server Documentation

## Table of Contents
1. [Quick Start](#quick-start)
2. [API Reference](#api-reference)
    - [Database Operations](#database-operations)
    - [Data Operations](#data-operations)
    - [Management Functions](#management-functions)
    - [Universal Command Interface](#universal-command-interface)
3. [Multi-language Examples](#multi-language-examples)
4. [Advanced Features](#advanced-features)
5. [Best Practices](#best-practices)

## Quick Start

### Start the Server
```bash
cargo run --bin server --release
# Default listening: http://127.0.0.1:14514
```

### Basic Workflow Example
```cmd
# 1. Create session and open database
curl -v -X POST http://127.0.0.1:14514/api/open ^
  -H "Content-Type: application/json" ^
  -d "{\"path\":\"test.db\"}"
# Don't forget to include the port number

# 2. Store data
curl -X POST http://127.0.0.1:14514/api/put ^
  -H "X-Session-ID: YOUR_SESSION_ID" ^
  -H "Content-Type: application/json" ^
  -d "[{\"key\":\"user1\",\"value\":\"Alice\"},{\"key\":\"user2\",\"value\":\"Bob\"}]"

# 3. Query data
curl "http://127.0.0.1:14514/api/get?key=user1" ^
  -H "X-Session-ID: YOUR_SESSION_ID"

# 4. Close connection
curl http://127.0.0.1:14514/api/close ^
  -H "X-Session-ID: YOUR_SESSION_ID"
```

## API Reference

### Database Operations

| Endpoint         | Method | Description      | Parameter Example          |
|------------------|--------|------------------|----------------------------|
| `/api/open`      | POST   | Open database    | `{"path": "data.db"}`      |
| `/api/close`     | GET    | Close database   | Requires Session ID        |
| `/api/current`   | GET    | Get current DB path | Requires Session ID     |

### Data Operations

| Endpoint     | Method | Description    | Parameter Example                    |
|--------------|--------|----------------|--------------------------------------|
| `/api/put`   | POST   | Batch write    | `[{"key":"k1","value":"v1"},...]`    |
| `/api/get`   | GET    | Single key query | `?key=target_key`                  |
| `/api/del`   | POST   | Delete key-value | `{"key":"target_key"}`            |

### Management Functions

| Endpoint         | Method | Description       | 
|------------------|--------|-------------------|
| `/api/compact`   | GET    | Compact database  |
| `/api/id/get`    | GET    | Get DB identifier |
| `/api/id/set`    | POST   | Set DB identifier |

### Universal Command Interface

#### `/api/execute` Endpoint
- **Method**: POST
- **Content-Type**: `text/plain`
- **Required Header**: `X-Session-ID`

**Supported Command Formats**:
```bash
# Data write
PUT "key1":"value1";

# Data query
GET WHERE KEY="key1";

# Data deletion
DEL WHERE KEY="key1";

# Database maintenance
COMPACT;
IDENTIFIER SET "new_name";
IDENTIFIER GET;
```

**Response Format**:
```json
{
  "status": "success|error",
  "result": "Execution result or error message"
}
```

## Multi-language Examples

### Python
```python
import requests

# Open database
res = requests.post("http://localhost:14514/api/open", 
                   json={"path": "demo.db"})
session_id = res.headers["X-Session-ID"]

# Execute command
requests.post("http://localhost:14514/api/execute",
             headers={"X-Session-ID": session_id},
             data='PUT "lang":"python";')
```

### JavaScript
```javascript
const axios = require('axios');

// Execute compound commands
axios.post('http://localhost:14514/api/execute',
  'PUT "counter":"1";\nGET WHERE KEY="counter";',
  {
    headers: {
      'X-Session-ID': sessionId,
      'Content-Type': 'text/plain'
    }
  }
);
```

### Java
```java
// Execute query command
String command = "GET WHERE KEY=\"ip\";";
HttpRequest request = HttpRequest.newBuilder()
    .uri(URI.create("http://localhost:14514/api/execute"))
    .header("X-Session-ID", sessionId)
    .header("Content-Type", "text/plain")
    .POST(HttpRequest.BodyPublishers.ofString(command))
    .build();
```

## Advanced Features

### Session Management
- Automatic session creation (returns `X-Session-ID` in response header)
- 30-minute inactivity timeout
- Background cleanup of idle sessions every 5 minutes

## Best Practices

1. **Session Management**:
    - Close unused sessions promptly
    - Send periodic requests to maintain session for long operations

2. **API Selection**:
    - Use dedicated endpoints for simple operations
    - Use `/api/execute` for complex batch processing

3. **Error Handling**:
    - Check HTTP status codes
    - Parse JSON error messages

4. **Performance Recommendations**:
    - Use array format for batch operations
    - Split large values into chunks for storage

> Note: All keys and values must be wrapped in double quotes, with no spaces around the colon separator
>
> When using the `/api/execute` route, separate multiple commands with semicolons
