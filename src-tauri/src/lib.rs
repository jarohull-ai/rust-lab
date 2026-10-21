use serde::Serialize;
use tauri::Manager;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiSession {
    base_url: String,
    token: String,
}

#[tauri::command]
fn api_session(session: tauri::State<'_, ApiSession>) -> ApiSession {
    session.inner().clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![api_session])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Bind an instance-specific loopback port before exposing the session to the UI.
            // Failure is fatal: a desktop instance must never share another process' API.
            let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
            listener.set_nonblocking(true)?;
            let addr = listener.local_addr()?;
            let state = api::AppState::default();
            app.manage(ApiSession {
                base_url: format!("http://{addr}"),
                token: state.api_token().to_string(),
            });

            tauri::async_runtime::spawn(async move {
                let router = api::app(state);
                match tokio::net::TcpListener::from_std(listener) {
                    Ok(listener) => {
                        log::info!("Polyglot Lab local API bound to an instance-specific loopback port");
                        if let Err(err) = axum::serve(listener, router).await {
                            log::error!("Polyglot Lab API server encountered error: {err}");
                        }
                    }
                    Err(err) => {
                        log::error!("Could not initialize the Polyglot Lab API listener: {err}");
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building tauri application");
}
