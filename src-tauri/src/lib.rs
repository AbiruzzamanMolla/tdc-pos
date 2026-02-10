mod commands;
mod db;
mod models;

use db::Database;
use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Initialize database
            let conn = db::init_db(app.handle())
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
            
            // Manage database connection in state
            app.manage(Database { conn: Mutex::new(conn) });
            
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_products,
            commands::create_product,
            commands::update_product,
            commands::delete_product,
            commands::create_purchase,
            commands::create_order
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
