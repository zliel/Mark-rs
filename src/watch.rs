use log::{error, info};
use notify::event::EventKind;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::broadcast;
use warp::Filter;

/// Starts the watch mode, which includes:
/// 1. A file watcher that triggers a rebuild on changes.
/// 2. A local web server that serves the output directory.
/// 3. A WebSocket server that sends reload signals to the browser.
///
/// # Arguments
/// * `input_dir` - Directory to watch for changes
/// * `output_dir` - Directory where output is served from
/// * `full_build_fn` - Called for initial build
/// * `incremental_build_fn` - Called with changed file paths for incremental rebuilds
pub fn start_watch_mode<F, I>(
    input_dir: String,
    output_dir: String,
    full_build_fn: F,
    incremental_build_fn: I,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    I: Fn(&[PathBuf]) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
        + Send
        + Sync
        + 'static,
{
    info!("Starting watch mode...");
    info!("Serving at http://localhost:3030");

    // Create a Tokio runtime for the async server and watcher components
    let rt = Runtime::new()?;

    rt.block_on(async {
        let (tx, _rx) = broadcast::channel::<String>(16);

        // Channel for file watcher events -> debouncer loop (now passes paths)
        let (notify_tx, mut notify_rx) = tokio::sync::mpsc::unbounded_channel::<PathBuf>();

        // Shared set to collect changed paths during debounce window
        let pending_paths: Arc<Mutex<HashSet<PathBuf>>> = Arc::new(Mutex::new(HashSet::new()));
        let pending_paths_watcher = Arc::clone(&pending_paths);

        // 1. Setup File Watcher
        let full_build_fn = Arc::new(full_build_fn);
        let incremental_build_fn = Arc::new(incremental_build_fn);
        let input_path = input_dir.clone();

        // Run initial full build
        if let Err(e) = full_build_fn() {
            error!("Initial build failed: {}", e);
        }

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        // Filter to only relevant events: Create, Modify (data changes)
                        // Ignore: Access, Remove, metadata-only changes
                        let dominated = matches!(
                            event.kind,
                            EventKind::Create(_)
                                | EventKind::Modify(notify::event::ModifyKind::Data(_))
                                | EventKind::Modify(notify::event::ModifyKind::Any)
                        );

                        if dominated {
                            for path in event.paths {
                                info!("File changed: {:?}", path);
                                // Add to pending set and notify
                                if let Ok(mut paths) = pending_paths_watcher.lock() {
                                    paths.insert(path.clone());
                                }
                                let _ = notify_tx.send(path);
                            }
                        }
                    }
                    Err(e) => error!("Watch error: {:?}", e),
                }
            },
            Config::default(), // Use native watcher (no poll interval for faster events on Windows)
        )?;

        watcher.watch(Path::new(&input_path), RecursiveMode::Recursive)?;

        // Spawn logic to handle debounced rebuilding
        let builder_tx = tx.clone();
        let incremental_build_fn_clone = Arc::clone(&incremental_build_fn);
        let pending_paths_builder = Arc::clone(&pending_paths);
        tokio::spawn(async move {
            loop {
                // Wait for the first notification
                if notify_rx.recv().await.is_none() {
                    break;
                }

                // Debounce: Wait a short buffer for more events to trickle in (e.g., "save all")
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Drain any pending events that happened during the wait
                while notify_rx.try_recv().is_ok() {}

                // Collect and clear pending paths
                let changed_paths: Vec<PathBuf> = {
                    let mut paths = pending_paths_builder.lock().unwrap();
                    let collected: Vec<PathBuf> = paths.drain().collect();
                    collected
                };

                if changed_paths.is_empty() {
                    continue;
                }

                info!("Rebuilding {} file(s)...", changed_paths.len());

                // Offload blocking build to a dedicated thread to avoid starving the async runtime
                let build_fn = Arc::clone(&incremental_build_fn_clone);
                let result = tokio::task::spawn_blocking(move || build_fn(&changed_paths)).await;

                match result {
                    Ok(Ok(_)) => {
                        info!("Rebuild successful.");
                        // Send reload signal
                        let receiver_count = builder_tx.receiver_count();
                        info!(
                            "Broadcasting reload to {} connected clients...",
                            receiver_count
                        );
                        match builder_tx.send("reload".to_string()) {
                            Ok(n) => info!("Reload signal sent to {} receivers", n),
                            Err(e) => error!("Failed to send reload signal: {:?}", e),
                        }
                    }
                    Ok(Err(e)) => {
                        error!("Rebuild failed: {}", e);
                    }
                    Err(e) => {
                        error!("Join error: {}", e);
                    }
                }
            }
        });

        // 2. Setup Web Server and WebSockets

        // Serve static files with no-cache headers to ensure browser sees updates
        let static_files = warp::fs::dir(output_dir.clone()).map(|reply| {
            warp::reply::with_header(reply, "Cache-Control", "no-store, must-revalidate")
        });

        // WebSocket route
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(warp::any().map(move || tx.clone()))
            .map(|ws: warp::ws::Ws, tx: broadcast::Sender<String>| {
                ws.on_upgrade(move |socket| handle_ws_client(socket, tx))
            });

        // Combine routes
        let routes = ws_route.or(static_files);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;

        Ok(())
    })
}

async fn handle_ws_client(ws: warp::ws::WebSocket, tx: broadcast::Sender<String>) {
    use futures_util::{SinkExt, StreamExt};

    info!(
        "WebSocket client connected. Total receivers: {}",
        tx.receiver_count()
    );
    let mut rx = tx.subscribe();
    info!(
        "Subscribed to broadcast channel. Now {} receivers.",
        tx.receiver_count()
    );

    // Split WebSocket into sender and receiver
    let (mut ws_tx, mut ws_rx) = ws.split();

    loop {
        tokio::select! {
            // Handle broadcast messages (reload signals)
            broadcast_result = rx.recv() => {
                match broadcast_result {
                    Ok(msg) if msg == "reload" => {
                        info!("Sending reload signal to browser...");
                        if let Err(e) = ws_tx.send(warp::ws::Message::text("reload")).await {
                            error!("Failed to send reload to WebSocket: {:?}", e);
                            break;
                        }
                        // Flush immediately for fastest delivery
                        if let Err(e) = ws_tx.flush().await {
                            error!("Failed to flush WebSocket: {:?}", e);
                            break;
                        }
                        info!("Reload signal delivered to browser.");
                    }
                    Ok(_) => {} // Ignore other messages
                    Err(e) => {
                        error!("Broadcast channel error: {:?}", e);
                        break;
                    }
                }
            }
            // Handle incoming WebSocket messages (keep connection alive)
            ws_msg = ws_rx.next() => {
                match ws_msg {
                    Some(Ok(msg)) => {
                        // Just acknowledge we received something - keeps connection alive
                        if msg.is_close() {
                            info!("WebSocket client disconnected.");
                            break;
                        }
                        // Ping/pong is handled automatically by warp
                    }
                    Some(Err(e)) => {
                        error!("WebSocket receive error: {:?}", e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended.");
                        break;
                    }
                }
            }
        }
    }
}
