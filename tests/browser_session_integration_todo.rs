// TODO: REFACTOR THIS TEST FOR ROBERT-WEBDRIVER
// This test was moved from robert-app/src-tauri/tests/browser_session_integration.rs
// It relies on robert-app-lib types that need to be adapted to robert-webdriver internal types.
// The logic for ephemeral profiles, session management, and launching needs to be ported.

/*
/// Integration tests for browser session lifecycle (Phase 2)
///
/// These tests verify the complete browser session workflow:
/// - Creating ephemeral profiles
/// - Launching browser sessions
/// - Querying session information
/// - Closing sessions and cleaning up resources
///
/// Note: These tests actually launch Chrome, so they require Chrome to be installed
/// and may take longer than unit tests.
use robert_app_lib::profiles::browser::{
    BrowserConfig, BrowserLauncher, BrowserProfile, SessionManager,
};

// ============================================================================
// Profile Lifecycle Tests
// ============================================================================

#[tokio::test]
async fn test_ephemeral_profile_creation_and_cleanup() {
    // Create ephemeral profile
    let profile = BrowserProfile::create_ephemeral().expect("Failed to create profile");

    // Verify profile is ephemeral
    assert!(profile.is_ephemeral());

    // Verify path exists
    assert!(profile.path().exists());

    // Get the path before cleanup for verification
    let profile_path = profile.path().to_path_buf();

    // Cleanup
    profile.cleanup().expect("Failed to cleanup profile");

    // Verify path no longer exists
    assert!(!profile_path.exists());
}

#[tokio::test]
async fn test_ephemeral_profile_id_uniqueness() {
    // Create multiple profiles
    let profile1 = BrowserProfile::create_ephemeral().expect("Failed to create profile1");
    let profile2 = BrowserProfile::create_ephemeral().expect("Failed to create profile2");

    // IDs should be unique
    assert_ne!(profile1.id(), profile2.id());

    // Paths should be unique
    assert_ne!(profile1.path(), profile2.path());

    // Cleanup
    profile1.cleanup().expect("Failed to cleanup profile1");
    profile2.cleanup().expect("Failed to cleanup profile2");
}

#[tokio::test]
async fn test_cleanup_orphaned_profiles() {
    use robert_app_lib::profiles::browser::profile::cleanup_orphaned_profiles;

    // Create some ephemeral profiles without cleaning them up
    let _profile1 = BrowserProfile::create_ephemeral().expect("Failed to create profile1");
    let _profile2 = BrowserProfile::create_ephemeral().expect("Failed to create profile2");

    // Cleanup orphaned profiles
    let count = cleanup_orphaned_profiles().expect("Failed to cleanup orphaned profiles");

    // Should have cleaned up at least our 2 profiles
    assert!(
        count >= 2,
        "Expected at least 2 profiles cleaned up, got {}",
        count
    );
}

// ============================================================================
// Session Manager Tests
// ============================================================================

#[tokio::test]
async fn test_session_manager_creation() {
    let manager = SessionManager::new();

    // Should start with no active sessions
    assert_eq!(manager.session_count().await, 0);
    assert!(!manager.has_active_sessions().await);

    // List should be empty
    let sessions = manager.list_sessions().await;
    assert_eq!(sessions.len(), 0);
}

#[tokio::test]
async fn test_session_manager_max_sessions_limit() {
    let manager = SessionManager::new();

    // Launch first session (should succeed)
    let config = BrowserConfig::new().headless(true);
    let result1 = manager.launch_session(config.clone()).await;

    match result1 {
        Ok(session_id) => {
            // First session should succeed
            assert_eq!(manager.session_count().await, 1);

            // Try to launch second session (should fail due to Phase 2 limit of 1)
            let result2 = manager.launch_session(config).await;
            assert!(
                result2.is_err(),
                "Expected second session to fail due to limit"
            );

            // Cleanup
            manager
                .close_session(&session_id)
                .await
                .expect("Failed to close session");
        }
        Err(e) => {
            // If first session fails, it's likely because Chrome isn't installed
            // or we're in a headless environment. This is okay for CI.
            eprintln!("Skipping test - Chrome launch failed: {}", e);
            eprintln!("This is expected in CI environments without Chrome");
        }
    }
}

#[tokio::test]
async fn test_session_lifecycle() {
    let manager = SessionManager::new();

    // Configure for headless mode (better for CI)
    let config = BrowserConfig::new().headless(true);

    // Launch session
    let result = manager.launch_session(config).await;

    match result {
        Ok(session_id) => {
            // Verify session was created
            assert_eq!(manager.session_count().await, 1);
            assert!(manager.has_active_sessions().await);

            // Get session info
            let info = manager
                .get_session_info(&session_id)
                .await
                .expect("Failed to get session info");

            assert_eq!(info.id, session_id);
            assert_eq!(info.profile_type, "ephemeral");
            assert!(info.profile_name.contains("Ephemeral"));

            // List sessions
            let sessions = manager.list_sessions().await;
            assert_eq!(sessions.len(), 1);
            assert_eq!(sessions[0].id, session_id);

            // Close session
            manager
                .close_session(&session_id)
                .await
                .expect("Failed to close session");

            // Verify session was removed
            assert_eq!(manager.session_count().await, 0);
            assert!(!manager.has_active_sessions().await);

            // Should not be able to get info for closed session
            let info_result = manager.get_session_info(&session_id).await;
            assert!(info_result.is_err(), "Expected error for closed session");
        }
        Err(e) => {
            // Chrome may not be available in CI
            eprintln!("Skipping test - Chrome launch failed: {}", e);
            eprintln!("This is expected in CI environments without Chrome");
        }
    }
}

#[tokio::test]
async fn test_close_all_sessions() {
    let manager = SessionManager::new();

    // For this test, we'll just verify the close_all_sessions method works
    // even when there are no sessions
    let count = manager
        .close_all_sessions()
        .await
        .expect("Failed to close all sessions");

    assert_eq!(count, 0, "Expected 0 sessions closed when none are active");
}

// ============================================================================
// Browser Launcher Tests
// ============================================================================

#[tokio::test]
async fn test_browser_launcher_ephemeral_launch() {
    let launcher = BrowserLauncher::new();

    // Configure for headless mode
    let config = BrowserConfig::new().headless(true);

    // Try to launch browser
    let result = launcher.launch_ephemeral(config).await;

    match result {
        Ok((driver, profile)) => {
            // Verify profile is ephemeral
            assert!(profile.is_ephemeral());
            assert!(profile.path().exists());

            // Verify driver is alive
            assert!(driver.is_alive().await);

            // Get profile path for cleanup verification
            let profile_path = profile.path().to_path_buf();

            // Cleanup
            drop(driver); // Drop driver to close browser
            profile.cleanup().expect("Failed to cleanup profile");

            // Verify profile directory was cleaned up
            assert!(!profile_path.exists());
        }
        Err(e) => {
            // Chrome may not be available in CI
            eprintln!("Skipping test - Chrome launch failed: {}", e);
            eprintln!("This is expected in CI environments without Chrome");
        }
    }
}

#[tokio::test]
async fn test_browser_config_options() {
    // Test headless configuration
    let headless_config = BrowserConfig::new().headless(true);
    assert!(headless_config.headless);
    assert!(!headless_config.no_sandbox);

    // Test no-sandbox configuration
    let no_sandbox_config = BrowserConfig::new().no_sandbox(true);
    assert!(!no_sandbox_config.headless);
    assert!(no_sandbox_config.no_sandbox);

    // Test combined configuration
    let combined_config = BrowserConfig::new().headless(true).no_sandbox(true);
    assert!(combined_config.headless);
    assert!(combined_config.no_sandbox);
}

#[tokio::test]
async fn test_browser_config_auto_ci() {
    // Clear CI env vars
    std::env::remove_var("CI");
    std::env::remove_var("GITHUB_ACTIONS");

    // In normal environment
    let config = BrowserConfig::auto_ci();
    assert!(!config.headless);
    assert!(!config.no_sandbox);

    // Set CI env var
    std::env::set_var("CI", "true");
    let config = BrowserConfig::auto_ci();
    assert!(config.headless);
    assert!(config.no_sandbox);

    // Cleanup
    std::env::remove_var("CI");
}
*/
