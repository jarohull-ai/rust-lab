#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Spawn background API server for the local desktop instance
            tauri::async_runtime::spawn(async move {
                let state = api::AppState::default();
                let router = api::app(state);
                let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3030));
                if let Ok(listener) = tokio::net::TcpListener::bind(addr).await {
                    let _ = axum::serve(listener, router).await;
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
