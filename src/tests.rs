#[cfg(test)]
mod test {

    use crate::{Handler, handle_request, parse_request, parse_request_line};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

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

        let (status, _reason, content_type, body) = handle_request(
            "GET",
            "/",
            temp_dir.path().to_str().unwrap(),
            &HashMap::new(),
        );
        println!(
            "Status: {}, Content-Type: {}, Body: {:?}",
            status, content_type, body
        );
        assert_eq!(status, 200);
        assert_eq!(content_type, "text/html");
        assert_eq!(body, Some(b"<h1>Hello, World!</h1>".to_vec()));
    }

    #[test]
    fn test_handle_request_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let (status, reason, content_type, body) = handle_request(
            "GET",
            "/nonexistent.html",
            temp_dir.path().to_str().unwrap(),
            &HashMap::new(),
        );
        assert_eq!(status, 404);
        assert_eq!(reason, "Not Found");
        assert_eq!(content_type, "text/plain");
        assert!(body.is_none());
    }

    #[test]
    fn test_handle_request_api_route() {
        let mut routes: HashMap<String, Handler> = HashMap::new();
        routes.insert("/api/hello".to_string(), || {
            (
                r#"{"message": "Hello, World!"}"#.to_string(),
                "application/json".to_string(),
            )
        });

        let (status, reason, content_type, body) = handle_request("GET", "/api/hello", "", &routes);
        assert_eq!(status, 200);
        assert_eq!(reason, "OK");
        assert_eq!(content_type, "application/json");
        assert_eq!(
            body,
            Some(r#"{"message": "Hello, World!"}"#.as_bytes().to_vec())
        );
    }
}
