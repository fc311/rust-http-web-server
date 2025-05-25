use std::collections::HashMap;

pub fn parse_request_line(line: &str) -> (&str, &str, &str) {
    let parts: Vec<&str> = line.split_whitespace().filter(|s| !s.is_empty()).collect();
    if parts.len() == 3 {
        (parts[0], parts[1], parts[2])
    } else {
        ("", "", "")
    }
}

pub fn parse_request(request: &str) -> (String, String, HashMap<String, String>) {
    // (String::new(), String::new(), HashMap::new())

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
}
