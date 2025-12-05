use clap::Parser;
use robert_webdriver::browser::chrome::ChromeDriver;
use robert_webdriver::cdp::{CdpExecutor, CdpScriptGenerator};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 9669)]
    port: u16,
}

#[derive(Debug, serde::Deserialize)]
struct InferenceRequest {
    prompt: String,
}

#[derive(Debug, serde::Serialize)]
struct InferenceResponse {
    status: String,
    message: String,
    script_steps: Option<usize>,
    execution_report: Option<serde_json::Value>,
}

// Shared state
struct AppState {
    driver: Mutex<Option<ChromeDriver>>,
    generator: CdpScriptGenerator,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    log::info!("Starting Robert Webdriver on port {}", args.port);

    // Initialize state
    let state = Arc::new(AppState {
        driver: Mutex::new(None),
        generator: CdpScriptGenerator::new(),
    });

    // Health check endpoint
    let health =
        warp::path("health").map(|| warp::reply::json(&serde_json::json!({ "status": "ok" })));

    // Inference endpoint
    let state_filter = warp::any().map(move || state.clone());

    let inference = warp::path("inference")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter)
        .and_then(handle_inference);

    let routes = health.or(inference);

    // Bind manually to handle "port in use" error gracefully
    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            log::info!("Listening on http://{}", addr);
            warp::serve(routes)
                .run_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                .await;
        }
        Err(e) => {
            log::error!("Failed to bind to port {}: {}", args.port, e);
            eprintln!(
                "Error: Port {} is already in use or unavailable.",
                args.port
            );
            std::process::exit(1);
        }
    }
}

async fn handle_inference(
    req: InferenceRequest,
    state: Arc<AppState>,
) -> Result<impl warp::Reply, warp::Rejection> {
    log::info!("Received inference request: {}", req.prompt);

    // 1. Ensure Chrome is running
    let mut driver_guard = state.driver.lock().await;

    // Check if alive, otherwise close and clear
    if let Some(driver) = driver_guard.as_ref() {
        if !driver.is_alive().await {
            log::warn!("Chrome session DEAD, restarting...");
            *driver_guard = None; // Drop dead driver
        }
    }

    // Launch if needed
    if driver_guard.is_none() {
        log::info!("Launching new Chrome session...");
        match ChromeDriver::launch_auto().await {
            Ok(d) => {
                log::info!("Chrome launched successfully.");
                *driver_guard = Some(d);
            }
            Err(e) => {
                log::error!("Failed to launch Chrome: {}", e);
                return Ok(warp::reply::json(&InferenceResponse {
                    status: "error".to_string(),
                    message: format!("Failed to launch Chrome: {}", e),
                    script_steps: None,
                    execution_report: None,
                }));
            }
        }
    }

    let driver = driver_guard.as_ref().unwrap();

    // Get page for execution
    let page = match driver.current_page().await {
        Ok(p) => p,
        Err(e) => {
            return Ok(warp::reply::json(&InferenceResponse {
                status: "error".to_string(),
                message: format!("Failed to get current page: {}", e),
                script_steps: None,
                execution_report: None,
            }));
        }
    };

    // 2. Generate Script
    let script_result = state.generator.generate(&req.prompt).await;

    match script_result {
        Ok(script) => {
            log::info!("Generated script with {} steps", script.cdp_commands.len());

            // 3. Execute Script
            let executor = CdpExecutor::new(page);
            match executor.execute_script(&script).await {
                Ok(report) => {
                    log::info!("Execution completed: {:?}", report);
                    Ok(warp::reply::json(&InferenceResponse {
                        status: "success".to_string(),
                        message: "Script generated and executed".to_string(),
                        script_steps: Some(script.cdp_commands.len()),
                        execution_report: serde_json::to_value(report).ok(),
                    }))
                }
                Err(e) => {
                    log::error!("Execution failed: {}", e);
                    Ok(warp::reply::json(&InferenceResponse {
                        status: "error".to_string(),
                        message: format!("Execution failed: {}", e),
                        script_steps: Some(script.cdp_commands.len()),
                        execution_report: None,
                    }))
                }
            }
        }
        Err(e) => {
            log::error!("Failed to generate script: {}", e);
            Ok(warp::reply::json(&InferenceResponse {
                status: "error".to_string(),
                message: format!("Generation failed: {}", e),
                script_steps: None,
                execution_report: None,
            }))
        }
    }
}
