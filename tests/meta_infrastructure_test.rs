//! Meta/Infrastructure Tests
//!
//! These tests verify that our testing infrastructure itself works correctly.
//! They test the test utilities, not the actual Chrome automation functionality.
//!
//! - Test server functionality
//! - Chrome installation detection
//! - Test isolation mechanisms

mod test_server;

use test_server::TestServer;

/// Meta test: Verify test server starts on a random port
#[tokio::test]
async fn meta_test_server_starts() {
    let server = TestServer::start().await;
    assert!(server.addr().port() > 0);
    println!("✅ Test server running on: {}", server.url());
}

/// Meta test: Verify test server serves correct HTML content
#[tokio::test]
async fn meta_test_server_serves_html() {
    let server = TestServer::start().await;
    server
        .wait_ready()
        .await
        .expect("Server failed to become ready");
    let url = server.url();

    // Make HTTP request to verify it works
    let response = reqwest::get(&url).await.unwrap();
    assert!(response.status().is_success());

    let body = response.text().await.unwrap();
    assert!(body.contains("Example Domain"));
    assert!(body.contains("<h1>"));

    println!("✅ Test server serves correct HTML");
}

/// Meta test: Verify multiple test servers get different ports for isolation
#[tokio::test]
async fn meta_test_multiple_servers_different_ports() {
    let server1 = TestServer::start().await;
    let server2 = TestServer::start().await;

    // Each server should have a different port
    assert_ne!(server1.addr().port(), server2.addr().port());

    println!("✅ Server 1: {}", server1.url());
    println!("✅ Server 2: {}", server2.url());
    println!("✅ Test isolation: different ports confirmed");
}

/// Meta test: Verify server wait_ready() detects when server is responsive
#[tokio::test]
async fn meta_test_server_wait_ready() {
    let server = TestServer::start().await;

    // This should succeed quickly since server starts immediately
    let result = server.wait_ready().await;
    assert!(result.is_ok(), "wait_ready() should succeed");

    println!("✅ Server readiness detection works");
}
