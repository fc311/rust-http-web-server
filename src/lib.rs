use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;

mod tests;

type Handler = fn() -> (String, String);

pub fn parse_request_line(line: &str) -> (&str, &str, &str) {
    let parts: Vec<&str> = line.split_whitespace().filter(|s| !s.is_empty()).collect();
    if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("", "", "")
    }
}

pub fn parse_request(request: &str) -> (String, String, HashMap<String, String>) {
    let mut lines = request.lines();
    let request_line = lines.next().unwrap_or("");
    let (method, path, _protocol) = parse_request_line(request_line);

    let mut headers = HashMap::new();
    for line in lines.take_while(|l| !l.is_empty()) {
        if let Some((key, value)) = line.split_once(": ") {
            headers.insert(key.to_string(), value.to_string());
        }
    }

    (method.to_string(), path.to_string(), headers)
}

pub fn handle_request(
    method: &str,
    path: &str,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
) -> (u16, String, String, Option<Vec<u8>>) {
    if method != "GET" {
        return (
            405,
            "Method Not Allowed".to_string(),
            "text/plain".to_string(),
            None,
        );
    }

    if let Some(handler) = routes.get(path) {
        let (body, content_type) = handler();
        return (200, "OK".to_string(), content_type, Some(body.into_bytes()));
    }

    let path = if path == "/" {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    // let file_path = format!("{}{}", base_dir, path.trim_start_matches('/'));
    let file_path = Path::new(base_dir).join(path).to_str().unwrap().to_string();

    println!("Looking for file: {}", file_path);

    if Path::new(&file_path).exists() {
        let contents = std::fs::read(&file_path).unwrap();

        let content_type = match Path::new(path).extension().and_then(|s| s.to_str()) {
            Some("html") => "text/html",
            Some("css") => "text/css",
            _ => "application/octet-stream",
        };

        (
            200,
            "OK".to_string(),
            content_type.to_string(),
            Some(contents),
        )
    } else {
        (404, "Not Found".to_string(), "text/plain".to_string(), None)
    }
}

pub fn handle_connection(
    mut stream: impl Read + Write,
    base_dir: &str,
    routes: &HashMap<String, Handler>,
) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    let (method, path, headers) = parse_request(&request);

    if !headers.contains_key("Host") && method != "" {
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
        stream.write_all(response.as_bytes()).unwrap();
        stream.flush().unwrap();
        return;
    }

    let (status, reason, content_type, body) = handle_request(&method, &path, base_dir, routes);
    let body = body.unwrap_or_default();

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
        status,
        reason,
        content_type,
        body.len()
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.write_all(&body).unwrap();
    stream.flush().unwrap();
}
