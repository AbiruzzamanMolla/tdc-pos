use crate::models::{Product, Purchase, PurchaseItem, Order, OrderItem, DashboardStats, SalesReportItem, InventoryReportItem, User, Expense};
use crate::db::Database;
use tauri::{State, AppHandle, Manager};
use rusqlite::params;
use std::collections::HashMap;

#[tauri::command]
pub fn get_products(db: State<Database>) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Fetch products with the first image if available
    let mut stmt = conn.prepare("
        SELECT p.id, p.product_name, p.product_code, p.category, p.brand, p.buying_price, p.default_selling_price, 
               p.stock_quantity, p.unit, p.tax_percentage, p.original_price, p.profit_percentage, p.facebook_link, p.product_link,
               p.created_at, p.updated_at, p.is_deleted,
               (SELECT image_path FROM product_images WHERE product_id = p.id LIMIT 1) as image_path
        FROM products p
        WHERE p.is_deleted = 0
    ").map_err(|e| e.to_string())?;
    
    let products_iter = stmt.query_map([], |row| {
        let image_path: Option<String> = row.get(17)?;
        let images = image_path.map(|path| vec![path]);

        Ok(Product {
            id: Some(row.get(0)?),
            product_name: row.get(1)?,
            product_code: row.get(2)?,
            category: row.get(3)?,
            brand: row.get(4)?,
            buying_price: row.get(5)?,
            default_selling_price: row.get(6)?,
            stock_quantity: row.get(7)?,
            unit: row.get(8)?,
            tax_percentage: row.get(9)?,
            original_price: row.get(10)?,
            profit_percentage: row.get(11)?,
            facebook_link: row.get(12)?,
            product_link: row.get(13)?,
            created_at: row.get(14)?,
            updated_at: row.get(15)?,
            is_deleted: row.get(16)?,
            images,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut products = Vec::new();
    for product in products_iter {
        products.push(product.map_err(|e| e.to_string())?);
    }
    
    Ok(products)
}

#[tauri::command]
pub fn get_product_images(product_id: i64, db: State<Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT image_path FROM product_images WHERE product_id = ?1").map_err(|e| e.to_string())?;
    let images_iter = stmt.query_map(params![product_id], |row| {
        Ok(row.get::<_, String>(0)?)
    }).map_err(|e| e.to_string())?;
    
    let mut images = Vec::new();
    for image in images_iter {
        let path = image.map_err(|e| e.to_string())?;
        images.push(path);
    }
    
    Ok(images)
}

#[tauri::command]
pub fn read_image_base64(path: String) -> Result<String, String> {
    let file_path = std::path::Path::new(&path);
    if !file_path.exists() {
        return Err(format!("File not found: {}", path));
    }
    
    let data = std::fs::read(file_path).map_err(|e| e.to_string())?;
    let base64_data = base64_encode(&data);
    
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("png").to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    };
    
    Ok(format!("data:{};base64,{}", mime, base64_data))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len() * 4 / 3 + 4);
    
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}

fn save_images(app: &AppHandle, images: Vec<String>) -> Result<Vec<String>, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let images_dir = app_dir.join("images");
    if !images_dir.exists() {
        std::fs::create_dir_all(&images_dir).map_err(|e| e.to_string())?;
    }

    let images_dir_str = images_dir.to_string_lossy().to_string();

    let mut saved_paths = Vec::new();
    for (i, path) in images.iter().enumerate() {
        // If already in our images dir, keep as-is
        if path.starts_with(&images_dir_str) {
            saved_paths.push(path.clone());
            continue;
        }

        let source_path = std::path::Path::new(path);
        if source_path.exists() {
            let ext = source_path.extension().and_then(|e| e.to_str()).unwrap_or("png");
            let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros();
            let new_filename = format!("{}_{}.{}", timestamp, i, ext);
            let dest_path = images_dir.join(new_filename);
            
            std::fs::copy(source_path, &dest_path).map_err(|e| e.to_string())?;
            saved_paths.push(dest_path.to_string_lossy().to_string());
        } else {
            // Path doesn't exist and not in our dir — skip silently
            saved_paths.push(path.clone());
        }
    }
    Ok(saved_paths)
}

#[tauri::command]
pub fn create_product(product: Product, images: Vec<String>, db: State<Database>, app: AppHandle) -> Result<i64, String> {
   let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
   let tx = conn.transaction().map_err(|e| e.to_string())?;
   
   tx.execute(
       "INSERT INTO products (product_name, product_code, category, brand, buying_price, default_selling_price, stock_quantity, unit, tax_percentage, original_price, profit_percentage, facebook_link, product_link) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
       params![
           product.product_name,
           product.product_code,
           product.category,
           product.brand,
           product.buying_price,
           product.default_selling_price,
           product.stock_quantity,
           product.unit,
           product.tax_percentage,
           product.original_price,
           product.profit_percentage,
           product.facebook_link,
           product.product_link
       ],
   ).map_err(|e| e.to_string())?;
   
   let product_id = tx.last_insert_rowid();

   // Copy images to AppData and insert paths
   let final_images = match save_images(&app, images) {
       Ok(paths) => paths,
       Err(e) => return Err(e),
   };

   for image_path in final_images {
        tx.execute(
            "INSERT INTO product_images (product_id, image_path) VALUES (?1, ?2)",
            params![product_id, image_path],
        ).map_err(|e| e.to_string())?;
   }

   tx.commit().map_err(|e| e.to_string())?;
   
   Ok(product_id)
}

#[tauri::command]
pub fn update_product(product: Product, images: Vec<String>, db: State<Database>, app: AppHandle) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Update Product Details
    tx.execute(
        "UPDATE products SET product_name = ?1, product_code = ?2, category = ?3, brand = ?4, buying_price = ?5, default_selling_price = ?6, stock_quantity = ?7, unit = ?8, tax_percentage = ?9, original_price = ?10, profit_percentage = ?11, facebook_link = ?12, product_link = ?13, updated_at = CURRENT_TIMESTAMP WHERE id = ?14",
        params![
            product.product_name,
            product.product_code,
            product.category,
            product.brand,
            product.buying_price,
            product.default_selling_price,
            product.stock_quantity,
            product.unit,
            product.tax_percentage,
            product.original_price,
            product.profit_percentage,
            product.facebook_link,
            product.product_link,
            product.id
        ],
    ).map_err(|e| e.to_string())?;

    // Handle Images: 
    // Simplest strategy: Delete all old images for this product and insert new set.
    // This assumes the frontend sends the COMPLETE list of images every time.
    if let Some(id) = product.id {
        tx.execute("DELETE FROM product_images WHERE product_id = ?1", params![id]).map_err(|e| e.to_string())?;
        
        // Save new images
        let final_images = match save_images(&app, images) {
             Ok(paths) => paths,
             Err(e) => return Err(e),
        };

        for image_path in final_images {
            tx.execute(
                "INSERT INTO product_images (product_id, image_path) VALUES (?1, ?2)",
                params![id, image_path],
            ).map_err(|e| e.to_string())?;
        }
    }
    
    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn delete_product(id: i64, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Soft delete
    conn.execute(
        "UPDATE products SET is_deleted = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![id],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn create_purchase(purchase: Purchase, items: Vec<PurchaseItem>, db: State<Database>) -> Result<i64, String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Insert Purchase
    tx.execute(
        "INSERT INTO purchases (supplier_name, supplier_phone, invoice_number, purchase_date, total_amount, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            purchase.supplier_name,
            purchase.supplier_phone,
            purchase.invoice_number,
            purchase.purchase_date,
            purchase.total_amount,
            purchase.notes
        ],
    ).map_err(|e| e.to_string())?;
    
    let purchase_id = tx.last_insert_rowid();
    
    // 2. Insert Items and Update Product
    for item in items {
        tx.execute(
            "INSERT INTO purchase_items (purchase_id, product_id, quantity, buying_price, extra_charge, subtotal, purchase_unit_cost) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                purchase_id,
                item.product_id,
                item.quantity,
                item.buying_price,
                item.extra_charge,
                item.subtotal,
                item.purchase_unit_cost
            ],
        ).map_err(|e| e.to_string())?;

        // 3. Update Product Stock and Average Cost using Weighted Average
        // old_quantity = existing product stock quantity
        // old_average_cost = existing product buying price
        let (old_quantity, old_average_cost): (f64, f64) = tx.query_row(
            "SELECT stock_quantity, buying_price FROM products WHERE id = ?1",
            params![item.product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;
        
        // old_total_value = old_quantity * old_average_cost
        // new_total_value = (quantity * unit_price) + extra_charge
        let old_total_value = old_quantity * old_average_cost;
        let new_total_value = (item.quantity * item.buying_price) + item.extra_charge;

        // updated_total_quantity = old_quantity + quantity
        // updated_total_value = old_total_value + new_total_value
        let updated_total_quantity = old_quantity + item.quantity;
        let updated_total_value = old_total_value + new_total_value;

        // new_average_buying_price = updated_total_value / updated_total_quantity
        let new_average_buying_price = if updated_total_quantity > 0.0 {
            updated_total_value / updated_total_quantity
        } else {
            item.purchase_unit_cost // Fallback if somehow quantity is 0
        };
        
        tx.execute(
            "UPDATE products SET stock_quantity = ?1, buying_price = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![updated_total_quantity, new_average_buying_price, item.product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(purchase_id)
}

#[tauri::command]
pub fn create_order(order: Order, items: Vec<OrderItem>, db: State<Database>) -> Result<i64, String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Insert Order
    tx.execute(
        "INSERT INTO orders (order_date, order_type, customer_name, customer_phone, customer_address, subtotal, extra_charge, delivery_charge, discount, grand_total, payment_method, notes) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            order.order_date,
            order.order_type,
            order.customer_name,
            order.customer_phone,
            order.customer_address,
            order.subtotal,
            order.extra_charge,
            order.delivery_charge,
            order.discount,
            order.grand_total,
            order.payment_method,
            order.notes
        ],
    ).map_err(|e| e.to_string())?;
    
    let order_id = tx.last_insert_rowid();
    
    // 2. Insert Items and Update Product
    for item in items {
        // Fetch current buying price for snapshot
        let buying_price: f64 = tx.query_row(
            "SELECT buying_price FROM products WHERE id = ?1",
            params![item.product_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, selling_price, subtotal, buying_price_snapshot) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                order_id,
                item.product_id,
                item.quantity,
                item.selling_price,
                item.subtotal,
                buying_price
            ],
        ).map_err(|e| e.to_string())?;
        
        // 3. Update Product Stock
        tx.execute(
            "UPDATE products SET stock_quantity = stock_quantity - ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![item.quantity, item.product_id],
        ).map_err(|e| e.to_string())?;

    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(order_id)
}

#[tauri::command]
pub fn get_purchases(db: State<Database>) -> Result<Vec<Purchase>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT purchase_id, supplier_name, supplier_phone, invoice_number, purchase_date, total_amount, notes, created_at FROM purchases ORDER BY purchase_date DESC").map_err(|e| e.to_string())?;
    
    let purchases_iter = stmt.query_map([], |row| {
        Ok(Purchase {
            purchase_id: Some(row.get(0)?),
            supplier_name: row.get(1)?,
            supplier_phone: row.get(2)?,
            invoice_number: row.get(3)?,
            purchase_date: row.get(4)?,
            total_amount: row.get(5)?,
            notes: row.get(6)?,
            created_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut purchases = Vec::new();
    for purchase in purchases_iter {
        purchases.push(purchase.map_err(|e| e.to_string())?);
    }
    
    Ok(purchases)
}

#[tauri::command]
pub fn get_orders(db: State<Database>) -> Result<Vec<Order>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT order_id, order_date, order_type, customer_name, customer_phone, customer_address, subtotal, extra_charge, delivery_charge, discount, grand_total, payment_method, notes FROM orders ORDER BY order_date DESC").map_err(|e| e.to_string())?;
    
    let orders_iter = stmt.query_map([], |row| {
        Ok(Order {
            order_id: Some(row.get(0)?),
            order_date: row.get(1)?,
            order_type: row.get(2)?,
            customer_name: row.get(3)?,
            customer_phone: row.get(4)?,
            customer_address: row.get(5)?,
            subtotal: row.get(6)?,
            extra_charge: row.get(7)?,
            delivery_charge: row.get(8)?,
            discount: row.get(9)?,
            grand_total: row.get(10)?,
            payment_method: row.get(11)?,
            notes: row.get(12)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut orders = Vec::new();
    for order in orders_iter {
        orders.push(order.map_err(|e| e.to_string())?);
    }
    
    Ok(orders)
}

#[tauri::command]
pub fn get_dashboard_stats(db: State<Database>) -> Result<DashboardStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // --- Sales Calculations ---
    let total_sales: f64 = conn.query_row("SELECT COALESCE(SUM(grand_total), 0) FROM orders", [], |row| row.get(0)).unwrap_or(0.0);
    let sales_today: f64 = conn.query_row("SELECT COALESCE(SUM(grand_total), 0) FROM orders WHERE date(order_date) = date('now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);
    let sales_month: f64 = conn.query_row("SELECT COALESCE(SUM(grand_total), 0) FROM orders WHERE strftime('%Y-%m', order_date) = strftime('%Y-%m', 'now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);
    let sales_year: f64 = conn.query_row("SELECT COALESCE(SUM(grand_total), 0) FROM orders WHERE strftime('%Y', order_date) = strftime('%Y', 'now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);

    // --- Purchases Calculations ---
    let total_purchases: f64 = conn.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM purchases", [], |row| row.get(0)).unwrap_or(0.0);
    let purchases_today: f64 = conn.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM purchases WHERE date(purchase_date) = date('now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);
    let purchases_month: f64 = conn.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM purchases WHERE strftime('%Y-%m', purchase_date) = strftime('%Y-%m', 'now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);
    let purchases_year: f64 = conn.query_row("SELECT COALESCE(SUM(total_amount), 0) FROM purchases WHERE strftime('%Y', purchase_date) = strftime('%Y', 'now', 'localtime')", [], |row| row.get(0)).unwrap_or(0.0);

    // --- Profit Calculations (Sales - COGS) ---
    // Helper to get COGS for a SQL condition
    let get_cogs = |condition: &str| -> f64 {
        let sql = format!("
            SELECT COALESCE(SUM(oi.quantity * oi.buying_price_snapshot), 0) 
            FROM order_items oi 
            JOIN orders o ON oi.order_id = o.order_id 
            WHERE {}", condition);
        conn.query_row(&sql, [], |row| row.get(0)).unwrap_or(0.0)
    };

    let total_cogs = get_cogs("1=1");
    let cogs_today = get_cogs("date(o.order_date) = date('now', 'localtime')");
    let cogs_month = get_cogs("strftime('%Y-%m', o.order_date) = strftime('%Y-%m', 'now', 'localtime')");
    let cogs_year  = get_cogs("strftime('%Y', o.order_date) = strftime('%Y', 'now', 'localtime')");

    let total_profit = total_sales - total_cogs;
    let profit_today = sales_today - cogs_today;
    let profit_month = sales_month - cogs_month;
    let profit_year  = sales_year - cogs_year;

    // --- Inventory & Meta ---
    let inventory_value: f64 = conn.query_row("SELECT COALESCE(SUM(stock_quantity * buying_price), 0) FROM products WHERE is_deleted = 0", [], |row| row.get(0)).unwrap_or(0.0);
    let low_stock_count: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE stock_quantity <= 5 AND is_deleted = 0", [], |row| row.get(0)).unwrap_or(0);
    let order_count: i64 = conn.query_row("SELECT COUNT(*) FROM orders", [], |row| row.get(0)).unwrap_or(0);
    let product_count: i64 = conn.query_row("SELECT COUNT(*) FROM products WHERE is_deleted = 0", [], |row| row.get(0)).unwrap_or(0);
    
    Ok(DashboardStats {
        sales_today,
        sales_month,
        sales_year,
        total_sales,
        purchases_today,
        purchases_month,
        purchases_year,
        total_purchases,
        profit_today,
        profit_month,
        profit_year,
        total_profit,
        inventory_value,
        low_stock_count,
        order_count,
        product_count,
    })
}

#[tauri::command]
pub fn get_sales_report(start_date: String, end_date: String, db: State<Database>) -> Result<Vec<SalesReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT 
            o.order_id, 
            o.order_date, 
            o.customer_name, 
            o.grand_total,
            COALESCE(o.discount, 0),
            COALESCE((SELECT COUNT(*) FROM order_items WHERE order_items.order_id = o.order_id), 0),
            (o.grand_total - COALESCE((SELECT SUM(quantity * buying_price_snapshot) FROM order_items WHERE order_items.order_id = o.order_id), 0)) as profit 
         FROM orders o 
         WHERE date(o.order_date) BETWEEN date(?1) AND date(?2)
         ORDER BY o.order_date DESC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map(params![start_date, end_date], |row| {
        Ok(SalesReportItem {
            order_id: row.get(0)?,
            date: row.get(1)?,
            customer: row.get(2)?,
            total: row.get(3)?,
            discount: row.get(4)?,
            items_count: row.get(5)?,
            profit: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}

#[tauri::command]
pub fn get_inventory_report(db: State<Database>) -> Result<Vec<InventoryReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare(
        "SELECT id, product_name, category, stock_quantity, unit, buying_price, default_selling_price, (stock_quantity * buying_price) as stock_value 
         FROM products 
         WHERE is_deleted = 0 
         ORDER BY stock_quantity ASC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(InventoryReportItem {
            id: row.get(0)?,
            name: row.get(1)?,
            category: row.get(2)?,
            stock: row.get(3)?,
            unit: row.get(4)?,
            cost_price: row.get(5)?,
            selling_price: row.get(6)?,
            stock_value: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for row in rows {
        items.push(row.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}

#[tauri::command]
pub fn backup_db(destination_path: String, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute_batch(&format!("VACUUM INTO '{}'", destination_path))
        .map_err(|e| format!("Backup failed: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn restore_db(source_path: String, app_handle: AppHandle) -> Result<(), String> {
    let app_dir = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let restore_path = app_dir.join("restore.db");
    
    if restore_path.exists() {
        std::fs::remove_file(&restore_path).map_err(|e| e.to_string())?;
    }
    
    std::fs::copy(&source_path, &restore_path).map_err(|e| format!("Failed to copy restore file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub fn list_backups(directory: String) -> Result<Vec<crate::models::BackupInfo>, String> {
    let path = std::path::Path::new(&directory);
    if !path.exists() {
        return Ok(Vec::new());
    }

    let entries = std::fs::read_dir(path).map_err(|e| e.to_string())?;
    let mut backups = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let metadata = entry.metadata().map_err(|e| e.to_string())?;
        
        if metadata.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".db") || name.ends_with(".bak") {
                backups.push(crate::models::BackupInfo {
                    name,
                    path: entry.path().to_string_lossy().to_string(),
                    size: metadata.len(),
                    created_at: format!("{:?}", metadata.created().unwrap_or(metadata.modified().unwrap())),
                });
            }
        }
    }
    
    // Sort by name (which usually includes date) descending
    backups.sort_by(|a, b| b.name.cmp(&a.name));
    
    Ok(backups)
}

#[tauri::command]
pub fn prune_backups(directory: String, keep_n: usize) -> Result<(), String> {
    let mut backups = list_backups(directory.clone())?;
    if backups.len() > keep_n {
        for backup in backups.drain(keep_n..) {
            let _ = std::fs::remove_file(backup.path);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn check_and_auto_backup(db: State<'_, Database>) -> Result<(), String> {
    let settings = get_settings(db.clone())?;
    
    let is_enabled = settings.get("auto_backup").map(|v| v == "true").unwrap_or(false);
    if !is_enabled { return Ok(()); }
    
    let backup_dir = settings.get("backup_dir");
    if backup_dir.is_none() || backup_dir.unwrap().is_empty() { return Ok(()); }
    let backup_dir = backup_dir.unwrap();

    let schedule = settings.get("backup_schedule").map(|v| v.as_str()).unwrap_or("daily");
    let keep_n = settings.get("keep_backups").and_then(|v| v.parse::<usize>().ok()).unwrap_or(5);
    
    let last_backup = settings.get("last_auto_backup_date");
    let now = chrono::Local::now();
    let now_str = now.format("%Y-%m-%d").to_string();

    let should_backup = match last_backup {
        Some(date) => {
            if schedule == "daily" {
                date != &now_str
            } else {
                // weekly - check if it's been 7 days or different week
                date != &now_str // simple check for now, can be improved
            }
        },
        None => true
    };

    if should_backup {
        let timestamp = now.format("%Y-%m-%d-%H-%M-%S").to_string();
        let name = format!("tdc-pos-auto-{}.db", timestamp);
        let path = std::path::Path::new(backup_dir).join(name);
        
        backup_db(path.to_string_lossy().to_string(), db.clone())?;
        prune_backups(backup_dir.clone(), keep_n)?;
        
        // Update last backup date
        update_settings(HashMap::from([("last_auto_backup_date".to_string(), now_str)]), db)?;
    }

    Ok(())
}

#[tauri::command]
pub fn get_settings(db: State<Database>) -> Result<HashMap<String, String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT key, value FROM settings").map_err(|e| e.to_string())?;
    let settings_iter = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?))
    }).map_err(|e| e.to_string())?;
    
    let mut settings = HashMap::new();
    for setting in settings_iter {
        let (key, value): (String, String) = setting.map_err(|e| e.to_string())?;
        settings.insert(key, value);
    }
    
    Ok(settings)
}

#[tauri::command]
pub fn update_settings(settings: HashMap<String, String>, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    for (key, value) in settings {
        tx.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_purchase_items(purchase_id: i64, db: State<Database>) -> Result<Vec<crate::models::PurchaseItemDetail>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("
        SELECT pi.id, pi.purchase_id, pi.product_id, p.product_name, pi.quantity, pi.buying_price, pi.extra_charge, pi.subtotal, pi.purchase_unit_cost
        FROM purchase_items pi
        JOIN products p ON pi.product_id = p.id
        WHERE pi.purchase_id = ?1
    ").map_err(|e| e.to_string())?;
    
    let items_iter = stmt.query_map(params![purchase_id], |row| {
        Ok(crate::models::PurchaseItemDetail {
            id: row.get(0)?,
            purchase_id: row.get(1)?,
            product_id: row.get(2)?,
            product_name: row.get(3)?,
            quantity: row.get(4)?,
            buying_price: row.get(5)?,
            extra_charge: row.get(6)?,
            subtotal: row.get(7)?,
            purchase_unit_cost: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for item in items_iter {
        items.push(item.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}

#[tauri::command]
pub fn delete_purchase(purchase_id: i64, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Get items to revert stock
    let items: Vec<(i64, f64)> = {
        let mut stmt = tx.prepare("SELECT product_id, quantity FROM purchase_items WHERE purchase_id = ?1").map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![purchase_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?;
        
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    // 2. Revert Stock (Subtract what was added)
    for (product_id, quantity) in items {
        tx.execute(
            "UPDATE products SET stock_quantity = stock_quantity - ?1 WHERE id = ?2",
            params![quantity, product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 3. Delete Items
    tx.execute("DELETE FROM purchase_items WHERE purchase_id = ?1", params![purchase_id]).map_err(|e| e.to_string())?;
    
    // 4. Delete Purchase
    tx.execute("DELETE FROM purchases WHERE purchase_id = ?1", params![purchase_id]).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_order(order_id: i64, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Get items to revert stock
    let items: Vec<(i64, f64)> = {
        let mut stmt = tx.prepare("SELECT product_id, quantity FROM order_items WHERE order_id = ?1").map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![order_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?;
        
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    // 2. Revert Stock (Add back what was sold)
    for (product_id, quantity) in items {
        tx.execute(
            "UPDATE products SET stock_quantity = stock_quantity + ?1 WHERE id = ?2",
            params![quantity, product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 3. Delete Items
    tx.execute("DELETE FROM order_items WHERE order_id = ?1", params![order_id]).map_err(|e| e.to_string())?;
    
    // 4. Delete Order
    tx.execute("DELETE FROM orders WHERE order_id = ?1", params![order_id]).map_err(|e| e.to_string())?;
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_purchase(purchase_id: i64, purchase: Purchase, items: Vec<PurchaseItem>, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Get old items to revert stock
    let old_items: Vec<(i64, f64, f64)> = {
        let mut stmt = tx.prepare("SELECT product_id, quantity, buying_price FROM purchase_items WHERE purchase_id = ?1").map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![purchase_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        }).map_err(|e| e.to_string())?;
        
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    // 2. Revert Stock and recalculate buying_price
    for (product_id, quantity, old_item_price) in old_items {
        let (current_stock, current_buying_price): (f64, f64) = tx.query_row(
            "SELECT stock_quantity, buying_price FROM products WHERE id = ?1",
            params![product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;

        let old_total_value = current_stock * current_buying_price;
        let reverted_stock = current_stock - quantity;
        
        let new_buying_price = if reverted_stock > 0.0 {
            (old_total_value - (quantity * old_item_price)) / reverted_stock
        } else {
            0.0
        };

        tx.execute(
            "UPDATE products SET stock_quantity = ?1, buying_price = ?2 WHERE id = ?3",
            params![reverted_stock, new_buying_price, product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 3. Delete old items
    tx.execute("DELETE FROM purchase_items WHERE purchase_id = ?1", params![purchase_id]).map_err(|e| e.to_string())?;
    
    // 4. Update Purchase record
    tx.execute(
        "UPDATE purchases SET supplier_name = ?1, supplier_phone = ?2, invoice_number = ?3, purchase_date = ?4, total_amount = ?5, notes = ?6 WHERE purchase_id = ?7",
        params![
            purchase.supplier_name,
            purchase.supplier_phone,
            purchase.invoice_number,
            purchase.purchase_date,
            purchase.total_amount,
            purchase.notes,
            purchase_id
        ],
    ).map_err(|e| e.to_string())?;

    // 5. Insert new items and applying their stock/cost changes
    for item in items {
        tx.execute(
            "INSERT INTO purchase_items (purchase_id, product_id, quantity, buying_price, extra_charge, subtotal, purchase_unit_cost) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                purchase_id,
                item.product_id,
                item.quantity,
                item.buying_price,
                item.extra_charge,
                item.subtotal,
                item.purchase_unit_cost
            ],
        ).map_err(|e| e.to_string())?;

        let (old_quantity, old_average_cost): (f64, f64) = tx.query_row(
            "SELECT stock_quantity, buying_price FROM products WHERE id = ?1",
            params![item.product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;
        
        let old_total_value = old_quantity * old_average_cost;
        let new_total_value = (item.quantity * item.buying_price) + item.extra_charge;

        let updated_total_quantity = old_quantity + item.quantity;
        let updated_total_value = old_total_value + new_total_value;

        let new_average_buying_price = if updated_total_quantity > 0.0 {
            updated_total_value / updated_total_quantity
        } else {
            item.purchase_unit_cost
        };
        
        tx.execute(
            "UPDATE products SET stock_quantity = ?1, buying_price = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![updated_total_quantity, new_average_buying_price, item.product_id],
        ).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn update_order(order_id: i64, order: Order, items: Vec<OrderItem>, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // 1. Get old items to revert stock
    let old_items: Vec<(i64, f64)> = {
        let mut stmt = tx.prepare("SELECT product_id, quantity FROM order_items WHERE order_id = ?1").map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![order_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        }).map_err(|e| e.to_string())?;
        
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| e.to_string())?);
        }
        result
    };

    // 2. Revert Stock (Add back what was sold)
    for (product_id, quantity) in old_items {
        tx.execute(
            "UPDATE products SET stock_quantity = stock_quantity + ?1 WHERE id = ?2",
            params![quantity, product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    // 3. Delete old items
    tx.execute("DELETE FROM order_items WHERE order_id = ?1", params![order_id]).map_err(|e| e.to_string())?;
    
    // 4. Update Order record
    tx.execute(
        "UPDATE orders SET order_date = ?1, order_type = ?2, customer_name = ?3, customer_phone = ?4, customer_address = ?5, subtotal = ?6, extra_charge = ?7, delivery_charge = ?8, discount = ?9, grand_total = ?10, payment_method = ?11, notes = ?12 WHERE order_id = ?13",
        params![
            order.order_date,
            order.order_type,
            order.customer_name,
            order.customer_phone,
            order.customer_address,
            order.subtotal,
            order.extra_charge,
            order.delivery_charge,
            order.discount,
            order.grand_total,
            order.payment_method,
            order.notes,
            order_id
        ],
    ).map_err(|e| e.to_string())?;

    // 5. Insert new items and updating product stock
    for item in items {
        let buying_price: f64 = tx.query_row(
            "SELECT buying_price FROM products WHERE id = ?1",
            params![item.product_id],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        
        tx.execute(
            "INSERT INTO order_items (order_id, product_id, quantity, selling_price, subtotal, buying_price_snapshot) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                order_id,
                item.product_id,
                item.quantity,
                item.selling_price,
                item.subtotal,
                buying_price
            ],
        ).map_err(|e| e.to_string())?;
        
        tx.execute(
            "UPDATE products SET stock_quantity = stock_quantity - ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
            params![item.quantity, item.product_id],
        ).map_err(|e| e.to_string())?;
    }
    
    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_order_items(order_id: i64, db: State<Database>) -> Result<Vec<crate::models::OrderItemDetail>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("
        SELECT oi.id, oi.order_id, oi.product_id, p.product_name, oi.quantity, oi.selling_price, oi.subtotal 
        FROM order_items oi
        JOIN products p ON oi.product_id = p.id
        WHERE oi.order_id = ?1
    ").map_err(|e| e.to_string())?;
    
    let items_iter = stmt.query_map(params![order_id], |row| {
        Ok(crate::models::OrderItemDetail {
            id: row.get(0)?,
            order_id: row.get(1)?,
            product_id: row.get(2)?,
            product_name: row.get(3)?,
            quantity: row.get(4)?,
            selling_price: row.get(5)?,
            subtotal: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut items = Vec::new();
    for item in items_iter {
        items.push(item.map_err(|e| e.to_string())?);
    }
    
    Ok(items)
}


#[tauri::command]
pub fn login(username: String, password: String, db: State<Database>) -> Result<User, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, username, role, created_at FROM users WHERE username = ?1 AND password = ?2")
        .map_err(|e| e.to_string())?;
    
    let user = stmt.query_row(params![username, password], |row| {
        Ok(User {
            id: Some(row.get(0)?),
            username: row.get(1)?,
            password: None,
            role: row.get(2)?,
            created_at: Some(row.get(3)?),
        })
    }).map_err(|_| "Invalid username or password".to_string())?;
    
    Ok(user)
}

#[tauri::command]
pub fn get_users(db: State<Database>) -> Result<Vec<User>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, username, role, created_at FROM users").map_err(|e| e.to_string())?;
    let user_iter = stmt.query_map([], |row| {
        Ok(User {
            id: Some(row.get(0)?),
            username: row.get(1)?,
            password: None,
            role: row.get(2)?,
            created_at: Some(row.get(3)?),
        })
    }).map_err(|e| e.to_string())?;
    
    let mut users = Vec::new();
    for user in user_iter {
        users.push(user.map_err(|e| e.to_string())?);
    }
    
    Ok(users)
}

#[tauri::command]
pub fn create_user(user: User, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
        "INSERT INTO users (username, password, role) VALUES (?1, ?2, ?3)",
        params![user.username, user.password.unwrap_or_default(), user.role],
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn delete_user(id: i64, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute("DELETE FROM users WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    
    Ok(())
}

#[tauri::command]
pub fn check_setup_required(db: State<Database>) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    Ok(count == 0)
}

#[tauri::command]
pub fn setup_admin(username: String, password: String, db: State<Database>) -> Result<User, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Double-check no users exist
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Err("Setup has already been completed".to_string());
    }

    conn.execute(
        "INSERT INTO users (username, password, role) VALUES (?1, ?2, 'super_admin')",
        params![username, password],
    ).map_err(|e| e.to_string())?;

    let id = conn.last_insert_rowid();
    Ok(User {
        id: Some(id),
        username,
        password: None,
        role: "super_admin".to_string(),
        created_at: None,
    })
}

#[tauri::command]
pub fn change_password(user_id: i64, current_password: String, new_password: String, is_super_admin: bool, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    if !is_super_admin {
        // Verify current password
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM users WHERE id = ?1 AND password = ?2",
            params![user_id, current_password],
            |row| row.get(0),
        ).map_err(|e| e.to_string())?;
        if count == 0 {
            return Err("Current password is incorrect".to_string());
        }
    }

    conn.execute(
        "UPDATE users SET password = ?1 WHERE id = ?2",
        params![new_password, user_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn update_user_role(user_id: i64, new_role: String, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE users SET role = ?1 WHERE id = ?2",
        params![new_role, user_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_product_purchase_history(product_id: i64, db: State<Database>) -> Result<Vec<crate::models::ProductPurchaseHistory>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("
        SELECT p.purchase_date, p.supplier_name, p.invoice_number, pi.quantity, pi.buying_price, pi.extra_charge, pi.subtotal, pi.purchase_unit_cost
        FROM purchase_items pi
        JOIN purchases p ON pi.purchase_id = p.purchase_id
        WHERE pi.product_id = ?1
        ORDER BY p.purchase_date DESC
    ").map_err(|e| e.to_string())?;
    
    let history_iter = stmt.query_map(params![product_id], |row| {
        Ok(crate::models::ProductPurchaseHistory {
            date: row.get(0)?,
            supplier_name: row.get(1)?,
            invoice_number: row.get(2)?,
            quantity: row.get(3)?,
            buying_price: row.get(4)?,
            extra_charge: row.get(5)?,
            subtotal: row.get(6)?,
            purchase_unit_cost: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;
    
    let mut history = Vec::new();
    for item in history_iter {
        history.push(item.map_err(|e| e.to_string())?);
    }
    
    Ok(history)
}
#[tauri::command]
pub fn get_product_stock_history(product_id: i64, db: State<Database>) -> Result<Vec<crate::models::StockMovement>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut movements = Vec::new();

    // 1. Fetch Purchases (IN)
    let mut stmt_in = conn.prepare("
        SELECT p.purchase_date, p.supplier_name, p.invoice_number, pi.quantity, pi.purchase_unit_cost
        FROM purchase_items pi
        JOIN purchases p ON pi.purchase_id = p.purchase_id
        WHERE pi.product_id = ?1
    ").map_err(|e| e.to_string())?;
    
    let in_rows = stmt_in.query_map(params![product_id], |row| {
        Ok(crate::models::StockMovement {
            date: row.get(0)?,
            movement_type: "IN".to_string(),
            entity_name: row.get(1)?,
            reference: row.get(2)?,
            quantity: row.get(3)?,
            price: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    for m in in_rows {
        movements.push(m.map_err(|e| e.to_string())?);
    }

    // 2. Fetch Sales (OUT)
    let mut stmt_out = conn.prepare("
        SELECT o.order_date, o.customer_name, o.order_id, oi.quantity, oi.selling_price
        FROM order_items oi
        JOIN orders o ON oi.order_id = o.order_id
        WHERE oi.product_id = ?1
    ").map_err(|e| e.to_string())?;
    
    let out_rows = stmt_out.query_map(params![product_id], |row| {
        Ok(crate::models::StockMovement {
            date: row.get(0)?,
            movement_type: "OUT".to_string(),
            entity_name: row.get(1)?,
            reference: Some(row.get::<_, i64>(2)?.to_string()),
            quantity: row.get(3)?,
            price: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    for m in out_rows {
        movements.push(m.map_err(|e| e.to_string())?);
    }

    // Sort by date DESC
    movements.sort_by(|a, b| b.date.cmp(&a.date));

    Ok(movements)
}

#[tauri::command]
pub fn log_activity(
    user_id: Option<i64>,
    username: String,
    action: String,
    entity_type: String,
    entity_id: Option<i64>,
    description: String,
    db: State<Database>
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO activity_logs (user_id, username, action, entity_type, entity_id, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![user_id, username, action, entity_type, entity_id, description],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_activity_logs(limit: i64, offset: i64, db: State<Database>) -> Result<Vec<crate::models::ActivityLog>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn.prepare(
        "SELECT id, user_id, username, action, entity_type, entity_id, description, created_at FROM activity_logs ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![limit, offset], |row| {
        Ok(crate::models::ActivityLog {
            id: Some(row.get(0)?),
            user_id: row.get(1)?,
            username: row.get(2)?,
            action: row.get(3)?,
            entity_type: row.get(4)?,
            entity_id: row.get(5)?,
            description: row.get(6)?,
            created_at: Some(row.get(7)?),
        })
    }).map_err(|e| e.to_string())?;

    let mut logs = Vec::new();
    for log in rows {
        logs.push(log.map_err(|e| e.to_string())?);
    }
    Ok(logs)
}

// --- Expenses ---
#[tauri::command]
pub fn create_expense(expense: Expense, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO expenses (expense_date, category, amount, notes) VALUES (?1, ?2, ?3, ?4)",
        params![expense.expense_date, expense.category, expense.amount, expense.notes],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_expenses(start_date: Option<String>, end_date: Option<String>, db: State<Database>) -> Result<Vec<Expense>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut query = "SELECT id, expense_date, category, amount, notes, created_at FROM expenses".to_string();
    let mut has_where = false;
    
    if start_date.is_some() && end_date.is_some() {
        query.push_str(" WHERE date(expense_date) >= date(?1) AND date(expense_date) <= date(?2)");
        has_where = true;
    }
    query.push_str(" ORDER BY expense_date DESC");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    
    let mut expenses = Vec::new();

    if start_date.is_some() && end_date.is_some() {
        let expense_iter = stmt.query_map(params![start_date.unwrap(), end_date.unwrap()], |row| {
            Ok(Expense {
                id: row.get(0)?,
                expense_date: row.get(1)?,
                category: row.get(2)?,
                amount: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;

        for exp_res in expense_iter {
            expenses.push(exp_res.map_err(|e| e.to_string())?);
        }
    } else {
        let expense_iter = stmt.query_map([], |row| {
            Ok(Expense {
                id: row.get(0)?,
                expense_date: row.get(1)?,
                category: row.get(2)?,
                amount: row.get(3)?,
                notes: row.get(4)?,
                created_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?;

        for exp_res in expense_iter {
            expenses.push(exp_res.map_err(|e| e.to_string())?);
        }
    }
    
    Ok(expenses)
}

#[tauri::command]
pub fn update_expense(id: i64, expense: Expense, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE expenses SET expense_date = ?1, category = ?2, amount = ?3, notes = ?4 WHERE id = ?5",
        params![expense.expense_date, expense.category, expense.amount, expense.notes, id],
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn delete_expense(id: i64, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM expenses WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn cleanup_database(
    clean_sales: bool, 
    clean_purchases: bool, 
    clean_products: bool, 
    clean_logs: bool, 
    clean_expenses: bool,
    db: State<Database>
) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    if clean_products {
        // If products are cleared, ALL history referencing them must go.
        // This is a hard reset of inventory and transactions.
        tx.execute("DELETE FROM product_images", []).map_err(|e| e.to_string())?;
        
        // Wipe all transaction tables completely
        tx.execute("DELETE FROM order_items", []).map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM orders", []).map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM purchase_items", []).map_err(|e| e.to_string())?;
        tx.execute("DELETE FROM purchases", []).map_err(|e| e.to_string())?;
        
        // Finally products
        tx.execute("DELETE FROM products", []).map_err(|e| e.to_string())?;
    } else {
        // If NOT cleaning products, check individual flags
        if clean_sales {
            tx.execute("DELETE FROM order_items", []).map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM orders", []).map_err(|e| e.to_string())?;
        }
    
        if clean_purchases {
            tx.execute("DELETE FROM purchase_items", []).map_err(|e| e.to_string())?;
            tx.execute("DELETE FROM purchases", []).map_err(|e| e.to_string())?;
        }
    }

    if clean_expenses {
        tx.execute("DELETE FROM expenses", []).map_err(|e| e.to_string())?;
    }

    if clean_logs {
        tx.execute("DELETE FROM activity_logs", []).map_err(|e| e.to_string())?;
    }

    tx.commit().map_err(|e| e.to_string())?;
    Ok(())
}
