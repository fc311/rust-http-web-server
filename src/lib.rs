use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, copy};
use std::io::{Read, Write};
use std::path::Path;

mod tests;

// Define a type alias for HTTP request handlers
// Each handler returns a tuple of (response_body: String, content_type: String)
pub type Handler = fn() -> (String, String);

/// Parses the first line of an HTTP request into its components
/// Returns a tuple of (HTTP_METHOD, REQUEST_PATH, HTTP_PROTOCOL)
/// Example: "GET /index.html HTTP/1.1" -> ("GET", "/index.html", "HTTP/1.1")
pub fn parse_request_line(line: &str) -> (&str, &str, &str) {
    let parts: Vec<&str> = line.split_whitespace().filter(|s| !s.is_empty()).collect();
    if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("", "", "") // Return empty strings if the request line is malformed
    }
}

/// Parses a complete HTTP request string into its components
/// Returns a tuple of (method: String, path: String, headers: HashMap)
/// Headers are stored as key-value pairs in a HashMap
pub fn parse_request(request: &str) -> (String, String, HashMap<String, String>) {
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let (method, path, _protocol) = parse_request_line(request_line);

    // Parse headers into a HashMap
    let mut headers = HashMap::new();
    for line in lines.take_while(|l| !l.is_empty()) {
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    (method.to_string(), path.to_string(), headers)
}

/// Handles an HTTP request and generates appropriate response
/// Parameters:
/// - method: HTTP method (GET, POST, etc.)
/// - path: Request path
/// - base_dir: Base directory for serving static files
/// - routes: HashMap of custom route handlers
/// Returns a tuple of (status_code, reason_phrase, content_type, response_writer_function)
pub fn handle_request(
    method: &str,
    path: &str,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
) -> (
    u16,
    String,
    String,
    Box<dyn Fn(&mut dyn Write) -> std::io::Result<()>>,
) {
    // Only handle GET requests, return 405 for other methods
    if method != "GET" {
        return (
            405,
            "Method Not Allowed".to_string(),
            "text/plain".to_string(),
            Box::new(|_| Ok(())),
        );
    }

    // Check if path matches any custom routes
    if let Some(handler) = routes.get(path) {
        let (body, content_type) = handler();
        let body_bytes = body.into_bytes();
        return (
            200,
            "OK".to_string(),
            content_type,
            Box::new(move |writer| {
                writer.write_all(&body_bytes)?;
                Ok(())
            }),
        );
    }

    // Handle root path by serving index.html
    let path = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    // Construct file path by joining base directory and request path
    let file_path = Path::new(base_dir).join(path).to_str().unwrap().to_string();

    // Serve static files if they exist
    if Path::new(&file_path).exists() {
        // Determine content type based on file extension
        let content_type = match Path::new(path).extension().and_then(|s| s.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            _ => "application/octet-stream",
        };

        (
            200,
            "OK".to_string(),
            content_type.to_string(),
            Box::new(move |writer| {
                let file = File::open(&file_path).unwrap();
                let mut reader = BufReader::new(file);
                copy(&mut reader, writer)?;
                Ok(())
            }),
        )
    } else {
        // Return 404 if file not found
        (
            404,
            "Not Found".to_string(),
            "text/plain".to_string(),
            Box::new(|_| Ok(())),
        )
    }
}

/// Handles an individual HTTP connection
/// Parameters:
/// - stream: The TCP stream for the connection (must implement Read + Write)
/// - base_dir: Base directory for serving static files
/// - routes: HashMap of custom route handlers
pub fn handle_connection(
    mut stream: impl Read + Write,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
) {
    // Read request into buffer
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    // Parse request
    let request = String::from_utf8_lossy(&buffer[..]);
    let (method, path, headers) = parse_request(&request);

    // Validate request has Host header (required by HTTP/1.1)
    if !headers.contains_key("Host") && method != "" {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    // Handle request and generate response
    let (status, reason, content_type, stream_fn) =
        handle_request(&method, &path, base_dir, routes);

    // Write response headers
    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\n\r\n",
        status, reason, content_type,
    );

    // Send response
    stream.write_all(response.as_bytes()).unwrap();
    stream_fn(&mut stream).unwrap();
    stream.flush().unwrap();
}
