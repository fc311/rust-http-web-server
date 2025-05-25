#[cfg(test)]
mod tests {

    use crate::{Handler, handle_connection, handle_request, parse_request, parse_request_line};
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::{Read, Write};
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

        let (status, _reason, content_type, _body) = handle_request(
            "GET",
            "/",
            temp_dir.path().to_str().unwrap(),
            &HashMap::new(),
        );

        assert_eq!(status, 200);
        assert_eq!(content_type, "text/html");
        // assert_eq!(body, Some(b"<h1>Hello, World!</h1>".to_vec()));
    }

    #[test]
    fn test_handle_request_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let (status, reason, content_type, _body) = handle_request(
            "GET",
            "/nonexistent.html",
            temp_dir.path().to_str().unwrap(),
            &HashMap::new(),
        );
        assert_eq!(status, 404);
        assert_eq!(reason, "Not Found");
        assert_eq!(content_type, "text/plain");
        // assert!(body.is_none());
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

        let (status, reason, content_type, _body) =
            handle_request("GET", "/api/hello", "", &routes);
        assert_eq!(status, 200);
        assert_eq!(reason, "OK");
        assert_eq!(content_type, "application/json");
        // assert_eq!(
        //     body,
        //     Some(r#"{"message": "Hello, World!"}"#.as_bytes().to_vec())
        // );
    }

    struct MockStream {
        read_data: Cursor<Vec<u8>>,
        write_data: Vec<u8>,
    }

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

    #[test]
    fn test_handle_connection_valid_request() {
        let mut routes: HashMap<String, Handler> = HashMap::new();
        routes.insert("/api/hello".to_string(), || {
            (
                r#"{"message": "Hello"}"#.to_string(),
                "application/json".to_string(),
            )
        });

        let request = b"GET /api/hello HTTP/1.1\r\nHost: localhost\r\n\r\n";
        let mut stream = MockStream {
            read_data: Cursor::new(request.to_vec()),
            write_data: Vec::new(),
        };

        handle_connection(&mut stream, "", &routes);

        let response = String::from_utf8_lossy(&stream.write_data);

        assert!(response.contains("HTTP/1.1 200 OK"));
        assert!(response.contains(r#"{"message": "Hello"}"#));
    }

    #[test]
    fn test_handle_request_stream_file() {
        let temp_dit = TempDir::new().unwrap();
        let file_path = temp_dit.path().join("index.html");
        File::create(&file_path)
            .unwrap()
            .write_all(b"<h1>Hello</h1>")
            .unwrap();

        let (status, _reason, content_type, stream_fn) = handle_request(
            "GET",
            "/",
            temp_dit.path().to_str().unwrap(),
            &HashMap::new(),
        );
        assert_eq!(status, 200);
        assert_eq!(content_type, "text/html");

        let mut output = Vec::new();
        stream_fn(&mut output).unwrap();
        assert_eq!(output, b"<h1>Hello</h1>");
    }
}
