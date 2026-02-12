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
            // Check for restore file
            let app_dir = app.path().app_data_dir().expect("failed to get app data dir");
            let db_path = app_dir.join("tdc-pos.db");
            let restore_path = app_dir.join("restore.db");

            if restore_path.exists() {
                // Perform restore
                 std::fs::copy(&restore_path, &db_path).expect("failed to restore database");
                 std::fs::remove_file(&restore_path).expect("failed to remove restore file");
                 println!("Database restored successfully.");
            }

            // Initialize database
            let conn = db::init_db(app.handle())
                .expect("failed to init db");
            
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
            commands::get_product_images,
            commands::read_image_base64,
            commands::create_purchase,
            commands::create_order,
            commands::get_purchases,
            commands::get_purchase_items,
            commands::get_order_items,
            commands::delete_purchase,
            commands::delete_order,
            commands::get_orders,
            commands::get_dashboard_stats,
            commands::get_sales_report,
            commands::get_inventory_report,
            commands::backup_db,
            commands::restore_db,
            commands::get_settings,
            commands::update_settings,
            commands::login,
            commands::get_users,
            commands::create_user,
            commands::delete_user,
            commands::list_backups,
            commands::prune_backups,
            commands::check_and_auto_backup,
            commands::get_product_purchase_history,
            commands::get_product_stock_history,
            commands::check_setup_required,
            commands::setup_admin,
            commands::change_password,
            commands::update_user_role
        ])


        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
