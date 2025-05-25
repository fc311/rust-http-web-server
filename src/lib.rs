use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

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
) -> (u16, String, String, Option<Vec<u8>>) {
    if method != "GET" {
        return (
            405,
            "Method Not Allowed".to_string(),
            "text/plain".to_string(),
            None,
        );
    }

    let path = if path == "/" { "index.html" } else { path };

    let file_path = format!("{}{}", base_dir, path);

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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_parse_request_line_valid() {
        let input = "GET / HTTP/1.1";

        let (method, path, protocol) = parse_request_line(input);
        assert_eq!(method, "GET");
        assert_eq!(path, "/");
        assert_eq!(protocol, "HTTP/1.1");
    }

    #[test]
    fn test_parse_request_valid() {
        let input = "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let (method, path, headers) = parse_request(input);
        assert_eq!(method, "GET");
        assert_eq!(path, "/");
        assert_eq!(headers.get("Host"), Some(&"localhost".to_string()));
    }
    #[test]
    fn test_parse_request_no_headers() {
        let input = "GET / HTTP/1.1\r\n\r\n";
        let (method, path, headers) = parse_request(input);
        assert_eq!(method, "GET");
        assert_eq!(path, "/");
        assert!(headers.is_empty());
    }

    #[test]
    fn test_handle_request_index_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("index.html");
        File::create(&file_path)
            .unwrap()
            .write_all(b"<h1>Hello, World!</h1>")
            .unwrap();
    }

    #[test]
    fn test_handle_request_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let (status, reason, content_type, body) = handle_request(
            "GET",
            "/nonexistent.html",
            temp_dir.path().to_str().unwrap(),
        );
        assert_eq!(status, 404);
        assert_eq!(reason, "Not Found");
        assert_eq!(content_type, "text/plain");
        assert!(body.is_none());
    }
}
