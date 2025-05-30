// Import required modules and types from our library and standard library
use rust_http_web_server::{Handler, handle_connection}; // Custom types and functions
use std::collections::HashMap;  // For storing route handlers
use std::net::TcpListener;     // For handling TCP connections
use std::thread;               // For multi-threading support

fn main() {
    // Create and bind TCP listener to localhost port 8080
    // unwrap() is used here as we want to panic if server fails to start
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    
    // Print server startup message with URL
    println!("Server running on http://127.0.0.1:8080");

    // Initialize route handler map
    // HashMap<String, Handler> maps URL paths to their handler functions
    let mut routes: HashMap<String, Handler> = HashMap::new();
    
    // Register API routes
    // This example adds a single route "/api/hello" that returns JSON
    routes.insert("/api/hello".to_string(), || {
        // Handler returns a tuple of (response_body, content_type)
        (
            r#"{"message": "Hello, API!"}"#.to_string(),  // JSON response
            "application/json".to_string(),                // Content-Type header
        )
    });

    // Main server loop
    // Continuously accept incoming connections
    for stream in listener.incoming() {
        // Safely unwrap the Result<TcpStream, Error>
        let stream = stream.unwrap();
        
        // Clone routes for the new thread
        // This is necessary because each thread needs its own copy
        let routes = routes.clone();
        
        // Spawn a new thread for each connection
        // This allows handling multiple connections concurrently
        thread::spawn(move || {
            // Handle the connection with:
            // - The TCP stream
            // - "static" as the base directory for static files
            // - Reference to the routes HashMap
            handle_connection(stream, "static", &routes);
        });
    }
}
