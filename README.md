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
