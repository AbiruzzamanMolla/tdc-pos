use rusqlite::{Connection, Result};
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct Database {
    pub conn: Mutex<Connection>,
}

pub fn init_db(app_handle: &AppHandle) -> Result<Connection> {
    // Get app data dir
    let app_dir = app_handle.path().app_data_dir().expect("failed to get app data dir");

    if !app_dir.exists() {
        std::fs::create_dir_all(&app_dir).expect("failed to create app data dir");
    }
    let db_path = app_dir.join("tdc-pos.db");
    
    let conn = Connection::open(db_path)?;
    
    // Create tables
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        
        CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_name TEXT NOT NULL,
            product_code TEXT UNIQUE,
            category TEXT,
            brand TEXT,
            buying_price REAL NOT NULL,
            default_selling_price REAL NOT NULL,
            stock_quantity REAL DEFAULT 0,
            unit TEXT,
            tax_percentage REAL DEFAULT 0,
            original_price REAL DEFAULT 0,
            profit_percentage REAL DEFAULT 0,
            facebook_link TEXT,
            product_link TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            is_deleted INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS product_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            FOREIGN KEY(product_id) REFERENCES products(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS purchases (
            purchase_id INTEGER PRIMARY KEY AUTOINCREMENT,
            supplier_name TEXT,
            supplier_phone TEXT,
            invoice_number TEXT,
            purchase_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            total_amount REAL,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS purchase_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            purchase_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity REAL NOT NULL,
            buying_price REAL NOT NULL,
            subtotal REAL NOT NULL,
            FOREIGN KEY(purchase_id) REFERENCES purchases(purchase_id),
            FOREIGN KEY(product_id) REFERENCES products(id)
        );

        CREATE TABLE IF NOT EXISTS customers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            customer_name TEXT NOT NULL,
            phone_number TEXT,
            address TEXT
        );

        CREATE TABLE IF NOT EXISTS orders (
            order_id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            order_type TEXT NOT NULL, -- local / online
            customer_name TEXT,
            customer_phone TEXT,
            customer_address TEXT,
            subtotal REAL,
            extra_charge REAL DEFAULT 0,
            delivery_charge REAL DEFAULT 0,
            discount REAL DEFAULT 0,
            grand_total REAL,
            payment_method TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS order_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            order_id INTEGER NOT NULL,
            product_id INTEGER NOT NULL,
            quantity REAL NOT NULL,
            selling_price REAL NOT NULL,
            subtotal REAL NOT NULL,
            buying_price_snapshot REAL,
            FOREIGN KEY(order_id) REFERENCES orders(order_id),
            FOREIGN KEY(product_id) REFERENCES products(id)
        );

        CREATE TABLE IF NOT EXISTS expenses (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            expense_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            category TEXT NOT NULL,
            amount REAL NOT NULL,
            notes TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'worker',
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS activity_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER,
            username TEXT NOT NULL,
            action TEXT NOT NULL,
            entity_type TEXT NOT NULL,
            entity_id INTEGER,
            description TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS messages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            conversation_id INTEGER NOT NULL,
            sender TEXT NOT NULL,
            content TEXT NOT NULL,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );

        -- No default users seeded; setup is done via frontend on first launch.
        "
    )?;

    {
        // Migrations for existing products table - check if columns exist first
        let mut stmt = conn.prepare("PRAGMA table_info(products)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut current_columns = std::collections::HashSet::new();
        for col_res in rows {
            current_columns.insert(col_res?);
        }

        if !current_columns.contains("original_price") {
            conn.execute("ALTER TABLE products ADD COLUMN original_price REAL DEFAULT 0", [])?;
        }
        if !current_columns.contains("facebook_link") {
            conn.execute("ALTER TABLE products ADD COLUMN facebook_link TEXT", [])?;
        }
        if !current_columns.contains("product_link") {
            conn.execute("ALTER TABLE products ADD COLUMN product_link TEXT", [])?;
        }
        if !current_columns.contains("profit_percentage") {
            conn.execute("ALTER TABLE products ADD COLUMN profit_percentage REAL DEFAULT 0", [])?;
        }
    }

    {
        // Migrations for purchase_items
        let mut stmt = conn.prepare("PRAGMA table_info(purchase_items)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        let mut current_columns = std::collections::HashSet::new();
        for col_res in rows {
            current_columns.insert(col_res?);
        }

        if !current_columns.contains("extra_charge") {
            conn.execute("ALTER TABLE purchase_items ADD COLUMN extra_charge REAL DEFAULT 0", [])?;
        }
        if !current_columns.contains("purchase_unit_cost") {
            conn.execute("ALTER TABLE purchase_items ADD COLUMN purchase_unit_cost REAL DEFAULT 0", [])?;
        }
    }


    
    Ok(conn)
}
