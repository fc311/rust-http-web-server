# rust-http-web-server

---

- start the HTTP web server:

```bash
cargo run
```

- test the server from CLI:

```bash
# Serves static/index.html
curl http://127.0.0.1:8080/

# Returns {"message": "Hello, API!"}
curl http://127.0.0.1:8080/api/hello
```

---

[TDD guide for HTTP Web Sever with Rust](https://grok.com/share/bGVnYWN5_b6ed2c43-56e1-459f-8f0a-45d8af979342)

---

# Implementation Documentation

## Table of Contents

- Overview
- Types
- Core Functions
- Content Type Mapping
- Error Handling
- Security Notes

## Overview

This library implements a basic HTTP web server with support for static file serving and custom route handlers. It provides functionality for parsing HTTP requests, handling different routes, and serving responses.

## Types

### Handler

```rust
pub type Handler = fn() -> (String, String);
```

A type alias representing route handler functions that return a tuple of:

- Response body (`String`)
- Content type (`String`)

## Core Functions

### parse_request_line

```rust
pub fn parse_request_line(line: &str) -> (&str, &str, &str)
```

**Purpose**: Parses the first line of an HTTP request.

**Parameters**:

- `line`: The request line string (e.g., "GET /path HTTP/1.1")

**Returns**:

- Tuple containing:
  - HTTP method (e.g., "GET")
  - Request path (e.g., "/path")
  - Protocol version (e.g., "HTTP/1.1")

**Error Handling**:

- Returns empty strings ("", "", "") if parsing fails

### parse_request

```rust
pub fn parse_request(request: &str) -> (String, String, HashMap<String, String>)
```

**Purpose**: Parses a complete HTTP request including headers.

**Parameters**:

- `request`: Complete HTTP request string

**Returns**:

- Tuple containing:
  - HTTP method
  - Request path
  - Headers map

### handle_request

```rust
pub fn handle_request(
    method: &str,
    path: &str,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
) -> (u16, String, String, Box<dyn Fn(&mut dyn Write) -> std::io::Result<()>>)
```

**Purpose**: Processes HTTP requests and generates appropriate responses.

**Parameters**:

- `method`: HTTP method
- `path`: Request path
- `base_dir`: Base directory for static files
- `routes`: Map of custom route handlers

**Returns**:

- Tuple containing:
  1. Status code (u16)
  2. Status reason phrase (String)
  3. Content type (String)
  4. Stream writer function (Box<dyn Fn>)

**Behavior**:

- Only handles GET requests (returns 405 for other methods)
- Checks for custom route handlers first
- Falls back to static file serving
- Returns 404 if resource not found

### handle_connection

```rust
pub fn handle_connection(
    mut stream: impl Read + Write,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
)
```

**Purpose**: Handles individual TCP connections and manages the request-response cycle.

**Parameters**:

- `stream`: TCP stream implementing Read + Write
- `base_dir`: Base directory for static files
- `routes`: Map of custom route handlers

**Behavior**:

1. Reads request from stream
2. Parses request headers
3. Validates presence of Host header
4. Processes request using `handle_request`
5. Writes response headers and body to stream

## Content Type Mapping

The server automatically determines content types for static files:

- `.html` → "text/html"
- `.css` → "text/css"
- Other extensions → "application/octet-stream"

## Error Handling

- 400 Bad Request: Missing Host header
- 404 Not Found: Missing resources
- 405 Method Not Allowed: Non-GET requests

## Security Notes

- Basic path sanitization implemented
- Directory traversal protection via `Path::new().join()`
- No authentication/authorization

---

# Test Documentation

## Overview

This documentation covers the test module for a Rust HTTP web server implementation. The tests verify various functionalities including request parsing, request handling, API routing, and file streaming.

## Test Structure

### Imports and Dependencies

```rust
use crate::{Handler, handle_connection, handle_request, parse_request, parse_request_line};
use std::collections::HashMap;
use std::fs::File;
use std::io::Cursor;
use std::io::{Read, Write};
use tempfile::TempDir;
```

## Test Cases

### 1. Request Line Parsing

#### `test_parse_request_line_valid()`

- **Purpose**: Validates parsing of basic HTTP request line
- **Tests**:
  - Method extraction (GET)
  - Path extraction (/)
  - Protocol version extraction (HTTP/1.1)
- **Expected Output**: Correctly parsed components of the request line

### 2. Request Parsing

#### `test_parse_request_valid()`

- **Purpose**: Tests parsing of complete HTTP requests with headers
- **Tests**:
  - Request line parsing
  - Header parsing
  - Host header validation
- **Expected Output**: Correctly parsed method, path, and headers map

#### `test_parse_request_no_headers()`

- **Purpose**: Verifies handling of requests without headers
- **Tests**: Request parsing with empty header section
- **Expected Output**: Valid method and path with empty headers map

### 3. Request Handling

#### `test_handle_request_index_file()`

- **Purpose**: Tests serving of index.html file
- **Setup**: Creates temporary directory with index.html
- **Tests**: GET request to root path
- **Expected Output**:
  - 200 status code
  - text/html content type

#### `test_handle_request_not_found()`

- **Purpose**: Validates 404 response for missing files
- **Tests**: GET request to non-existent path
- **Expected Output**:
  - 404 status code
  - "Not Found" reason
  - text/plain content type

### 4. API Routing

#### `test_handle_request_api_route()`

- **Purpose**: Tests API route handling
- **Setup**: Registers /api/hello route handler
- **Tests**: GET request to API endpoint
- **Expected Output**:
  - 200 status code
  - application/json content type
  - Correct JSON response

### 5. Connection Handling

#### Mock Stream Implementation

```rust
struct MockStream {
    read_data: Cursor<Vec<u8>>,
    write_data: Vec<u8>,
}
```

- **Purpose**: Simulates TCP stream for testing
- **Capabilities**:
  - Read simulation
  - Write capture
  - Flush operation

#### `test_handle_connection_valid_request()`

- **Purpose**: End-to-end testing of connection handling
- **Setup**:
  - Configures API route
  - Creates mock stream with request
- **Tests**: Complete request-response cycle
- **Expected Output**:
  - 200 OK response
  - Valid JSON payload

### 6. File Streaming

#### `test_handle_request_stream_file()`

- **Purpose**: Tests file streaming functionality
- **Setup**: Creates temporary file with HTML content
- **Tests**: File streaming to response
- **Expected Output**:
  - 200 status code
  - text/html content type
  - Correct file contents in stream

## Testing Utilities

### MockStream Implementation

```rust
impl Read for MockStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.read_data.read(buf)
    }
}

impl Write for MockStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.write_data.extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
```

- **Purpose**: Provides mock implementation for testing I/O operations
- **Features**:
  - Simulated read operations
  - Captured write operations
  - No-op flush implementation

## Test Coverage

The test suite covers:

- Request parsing
- Header handling
- File serving
- API routing
- Error handling
- Stream handling
- File streaming

---

# Documentation for main.rs

## Overview

This file implements a multi-threaded HTTP web server in Rust that supports both API routes and static file serving. The server runs on `localhost:8080` and demonstrates basic routing and concurrent connection handling.

## Components

### Imports

```rust
use rust_http_web_server::{Handler, handle_connection}; // Custom handler types and connection logic
use std::collections::HashMap;  // Route storage
use std::net::TcpListener;     // TCP server functionality
use std::thread;               // Multi-threading support
```

### Main Function Structure

The `main()` function serves as the entry point and performs these key operations:

1. **Server Initialization**

   - Creates a TCP listener bound to `127.0.0.1:8080`
   - Provides immediate feedback on server startup

2. **Route Configuration**

   - Initializes a `HashMap` to store route handlers
   - Routes are mapped as `String` (path) to `Handler` (function) pairs

3. **API Route Registration**

   - Demonstrates route registration with `/api/hello` endpoint
   - Returns JSON response with appropriate content type

4. **Connection Handling**
   - Implements an infinite loop to accept incoming connections
   - Spawns a new thread for each connection
   - Passes route information to connection handlers

## Detailed Code Analysis

### Server Binding

```rust
let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
```

- Binds server to localhost port 8080
- Uses `unwrap()` to handle binding failures with immediate panic

### Route Handler Setup

```rust
let mut routes: HashMap<String, Handler> = HashMap::new();
```

- Creates a mutable HashMap for storing route handlers
- `Handler` is a custom type (likely a function type) defined in the project's lib.rs

### API Route Definition

```rust
routes.insert("/api/hello".to_string(), || {
    (
        r#"{"message": "Hello, API!"}"#.to_string(),
        "application/json".to_string(),
    )
});
```

- Registers a simple JSON API endpoint
- Handler returns a tuple containing:
  - Response body (JSON string)
  - Content-Type header value

### Connection Processing

```rust
for stream in listener.incoming() {
    let stream = stream.unwrap();
    let routes = routes.clone();

    thread::spawn(move || {
        handle_connection(stream, "static", &routes);
    });
}
```

- Implements concurrent connection handling
- Each connection runs in its own thread
- Routes are cloned for thread safety
- Static file directory is set to "static"

## Important Notes

1. **Thread Safety**

   - Route handlers must be `Clone`-able
   - Each connection gets its own copy of routes

2. **Error Handling**

   - Server binding uses `unwrap()` - fails fast on startup errors
   - Connection acceptance also uses `unwrap()` - individual connection failures don't crash server

3. **Static File Serving**
   - Server expects static files in a "static" directory
   - Directory path is relative to server execution path

## Usage Example

1. Start the server:

   ```bash
   cargo run
   ```

2. Access the API endpoint:
   ```bash
   curl http://localhost:8080/api/hello
   ```
   Expected response:
   ```json
   { "message": "Hello, API!" }
   ```

## Dependencies

- Custom `rust_http_web_server` library
- Standard library components (`std::collections`, `std::net`, `std::thread`)

---
