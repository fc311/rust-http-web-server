use rust_http_web_server::{Handler, handle_connection};
use std::collections::HashMap;
use std::net::TcpListener;
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Server running on http://127.0.0.1:8080");

    let mut routes: HashMap<String, Handler> = HashMap::new();
    routes.insert("/api/hello".to_string(), || {
        (
            r#"{"message": "Hello, API!"}"#.to_string(),
            "application/json".to_string(),
        )
    });

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let routes = routes.clone();
        thread::spawn(move || {
            handle_connection(stream, "static", &routes);
        });
    }
}
