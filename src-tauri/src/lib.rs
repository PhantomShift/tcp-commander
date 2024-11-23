use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_store::StoreExt;

#[tauri::command]
async fn ask(app: tauri::AppHandle, msg: &str) -> Result<bool, ()> {
    let confirm = app
        .dialog()
        .message(msg)
        .kind(tauri_plugin_dialog::MessageDialogKind::Info)
        .title("Confirm")
        .buttons(tauri_plugin_dialog::MessageDialogButtons::YesNo)
        .blocking_show();

    Ok(confirm)
}

#[tauri::command]
fn initialize(app: tauri::AppHandle) -> AppData {
    println!("Initialize command called");
    app.state::<AppData>().inner().clone()
}

#[tauri::command]
async fn store_set(
    app: tauri::AppHandle,
    path: &str,
    key: &str,
    value: &str,
) -> Result<(), String> {
    app.store(path).unwrap().set(key, value);
    Ok(())
}

#[tauri::command]
async fn store_delete(app: tauri::AppHandle, path: &str, key: &str) -> Result<(), String> {
    app.store(path).unwrap().delete(key);
    Ok(())
}

#[tauri::command]
async fn store_get(app: tauri::AppHandle, path: &str, key: &str) -> Result<Option<String>, String> {
    Ok(app.store(path).unwrap().get(key).map(|v| v.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppData {
    pub last_address: Option<String>,
    pub last_port: Option<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_tcp_client::init())
        .setup(|app| {
            let store = app.store("store.json")?;
            if store.is_empty() {
                store.set("commands", json!([]));
            }

            let last_address = store
                .get("last_address")
                .map(|v| v.as_str().map(str::to_string))
                .flatten();
            let last_port = store
                .get("last_port")
                .map(|v| v.as_str().map(str::to_string))
                .flatten();

            app.manage(AppData {
                last_address,
                last_port,
            });

            Ok(())
        })
        // On shutdown
        // .on_menu_event(|app, event| {
        //     if event.id() == "quit" {
        //     }
        // })
        .invoke_handler(tauri::generate_handler![
            ask,
            initialize,
            store_set,
            store_delete,
            store_get
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
