//! E2E Tests for WASI HTTP Components
//!
//! These tests spawn `wasmtime serve` and make real HTTP requests.
//!
//! ## Prerequisites
//!
//! Build and compose components first:
//!
//! ```bash
//! # Build bridge
//! cd mik-bridge && cargo component build --release
//!
//! # Build handler
//! cd examples/hello-world && cargo component build --release
//!
//! # Compose (from repo root)
//! wac plug target/wasm32-wasip2/release/mik_bridge.wasm \
//!     --plug target/wasm32-wasip2/release/hello_world.wasm \
//!     -o tests-integration/fixtures/hello-world-service.wasm
//! ```

use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command};
use std::thread;
use std::time::Duration;

/// Find an available port for the test server.
fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

/// Test server wrapper that cleans up on drop.
struct TestServer {
    process: Child,
    port: u16,
}

impl TestServer {
    /// Start wasmtime serve with the given WASM component.
    fn start(wasm_path: &std::path::Path) -> anyhow::Result<Self> {
        let port = find_available_port();

        let process = Command::new("wasmtime")
            .args([
                "serve",
                "-S",
                "cli=y", // Enable CLI support
                "--addr",
                &format!("127.0.0.1:{port}"),
                wasm_path.to_str().unwrap(),
            ])
            .spawn()?;

        // Wait for server to start
        thread::sleep(Duration::from_millis(500));

        Ok(Self { process, port })
    }

    fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

fn get_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

// =============================================================================
// hello-world E2E Tests
// =============================================================================

#[test]
#[ignore = "requires pre-built WASM components - run with: cargo test --ignored"]
fn test_hello_world_home() {
    let wasm_path = get_fixture_path("hello-world-service.wasm");
    if !wasm_path.exists() {
        eprintln!("Skipping: {} not found. Build with:", wasm_path.display());
        eprintln!("  cd mik-bridge && cargo component build --release");
        eprintln!("  cd examples/hello-world && cargo component build --release");
        eprintln!("  wac plug ... -o tests-integration/fixtures/hello-world-service.wasm");
        return;
    }

    let server = TestServer::start(&wasm_path).expect("Failed to start server");

    let response = ureq::get(&format!("{}/", server.base_url()))
        .call()
        .expect("Request failed");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.header("content-type"),
        Some("application/json")
    );

    let json: serde_json::Value = response.into_json().expect("Failed to parse JSON");
    assert!(json["message"].is_string());
}

#[test]
#[ignore = "requires pre-built WASM components"]
fn test_hello_world_hello_with_name() {
    let wasm_path = get_fixture_path("hello-world-service.wasm");
    if !wasm_path.exists() {
        return;
    }

    let server = TestServer::start(&wasm_path).expect("Failed to start server");

    let response = ureq::get(&format!("{}/hello/Claude", server.base_url()))
        .call()
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.into_json().expect("Failed to parse JSON");
    assert_eq!(json["name"], "Claude");
}

#[test]
#[ignore = "requires pre-built WASM components"]
fn test_hello_world_search_with_query() {
    let wasm_path = get_fixture_path("hello-world-service.wasm");
    if !wasm_path.exists() {
        return;
    }

    let server = TestServer::start(&wasm_path).expect("Failed to start server");

    let response = ureq::get(&format!("{}/search?q=rust&page=2", server.base_url()))
        .call()
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.into_json().expect("Failed to parse JSON");
    assert_eq!(json["query"], "rust");
    assert_eq!(json["page"], 2);
}

#[test]
#[ignore = "requires pre-built WASM components"]
fn test_hello_world_404() {
    let wasm_path = get_fixture_path("hello-world-service.wasm");
    if !wasm_path.exists() {
        return;
    }

    let server = TestServer::start(&wasm_path).expect("Failed to start server");

    let response = ureq::get(&format!("{}/nonexistent", server.base_url())).call();

    // ureq returns Err for non-2xx status codes
    match response {
        Ok(_) => panic!("Expected 404"),
        Err(ureq::Error::Status(code, _)) => assert_eq!(code, 404),
        Err(e) => panic!("Unexpected error: {e}"),
    }
}

#[test]
#[ignore = "requires pre-built WASM components"]
fn test_hello_world_echo_post() {
    let wasm_path = get_fixture_path("hello-world-service.wasm");
    if !wasm_path.exists() {
        return;
    }

    let server = TestServer::start(&wasm_path).expect("Failed to start server");

    let response = ureq::post(&format!("{}/echo", server.base_url()))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({"message": "Hello from test"}))
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let json: serde_json::Value = response.into_json().expect("Failed to parse JSON");
    assert!(json["echo"].is_string() || json["received"].is_object());
}
