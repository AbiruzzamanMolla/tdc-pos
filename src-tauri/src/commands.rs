use crate::models::{Product, Purchase, PurchaseItem, Order, OrderItem, DashboardStats, SalesReportItem, InventoryReportItem};
use crate::db::Database;
use tauri::{State, AppHandle, Manager};
use rusqlite::params;
use std::collections::HashMap;

#[tauri::command]
pub fn get_products(db: State<Database>) -> Result<Vec<Product>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    let mut stmt = conn.prepare("SELECT id, product_name, product_code, category, brand, buying_price, default_selling_price, stock_quantity, unit, tax_percentage, created_at, updated_at, is_deleted FROM products WHERE is_deleted = 0").map_err(|e| e.to_string())?;
    
    let products_iter = stmt.query_map([], |row| {
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
            created_at: row.get(10)?,
            updated_at: row.get(11)?,
            is_deleted: row.get(12)?,
            images: None, // Will populate separately if needed, or keeping empty for list performance?
                          // For now, let's just initialize it as None or empty Vec.
                          // If we want to show images in list, we should fetch them.
                          // But creating N+1 query problem. 
                          // Better to have a separate command `get_product_images` or fetch in a joined query if needed.
                          // Let's populate with empty for `get_products` to keep it fast, 
                          // AND add `get_product_details` that includes images?
                          // Or just `get_product_images`.
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
        Ok(row.get(0)?)
    }).map_err(|e| e.to_string())?;
    
    let mut images = Vec::new();
    for image in images_iter {
        images.push(image.map_err(|e| e.to_string())?);
    }
    
    Ok(images)
}

#[tauri::command]
pub fn create_product(product: Product, images: Vec<String>, db: State<Database>) -> Result<i64, String> {
   let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
   let tx = conn.transaction().map_err(|e| e.to_string())?;
   
   tx.execute(
       "INSERT INTO products (product_name, product_code, category, brand, buying_price, default_selling_price, stock_quantity, unit, tax_percentage) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
       params![
           product.product_name,
           product.product_code,
           product.category,
           product.brand,
           product.buying_price,
           product.default_selling_price,
           product.stock_quantity,
           product.unit,
           product.tax_percentage
       ],
   ).map_err(|e| e.to_string())?;
   
   let product_id = tx.last_insert_rowid();

   // Insert images
   for image_path in images {
        tx.execute(
            "INSERT INTO product_images (product_id, image_path) VALUES (?1, ?2)",
            params![product_id, image_path],
        ).map_err(|e| e.to_string())?;
   }

   tx.commit().map_err(|e| e.to_string())?;
   
   Ok(product_id)
}

#[tauri::command]
pub fn update_product(product: Product, images: Vec<String>, db: State<Database>) -> Result<(), String> {
    let mut conn = db.conn.lock().map_err(|e| e.to_string())?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;
    
    // Update Product Details
    tx.execute(
        "UPDATE products SET product_name = ?1, product_code = ?2, category = ?3, brand = ?4, buying_price = ?5, default_selling_price = ?6, stock_quantity = ?7, unit = ?8, tax_percentage = ?9, updated_at = CURRENT_TIMESTAMP WHERE id = ?10",
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
            product.id
        ],
    ).map_err(|e| e.to_string())?;

    // Handle Images: 
    // Simplest strategy: Delete all old images for this product and insert new set.
    // This assumes the frontend sends the COMPLETE list of images every time.
    if let Some(id) = product.id {
        tx.execute("DELETE FROM product_images WHERE product_id = ?1", params![id]).map_err(|e| e.to_string())?;
        
        for image_path in images {
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
            "INSERT INTO purchase_items (purchase_id, product_id, quantity, buying_price, subtotal) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                purchase_id,
                item.product_id,
                item.quantity,
                item.buying_price,
                item.subtotal
            ],
        ).map_err(|e| e.to_string())?;

        // 3. Update Product Stock and Average Cost
        // Fetch current stock and price
        let (current_stock, current_price): (f64, f64) = tx.query_row(
            "SELECT stock_quantity, buying_price FROM products WHERE id = ?1",
            params![item.product_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).map_err(|e| e.to_string())?;
        
        let new_stock = current_stock + item.quantity;
        let new_price = if new_stock > 0.0 {
            ((current_stock * current_price) + (item.quantity * item.buying_price)) / new_stock
        } else {
            item.buying_price
        };
        
        tx.execute(
            "UPDATE products SET stock_quantity = ?1, buying_price = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
            params![new_stock, new_price, item.product_id],
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
    
    // Total Sales
    let total_sales: f64 = conn.query_row(
        "SELECT COALESCE(SUM(grand_total), 0) FROM orders",
        [],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    // Sales Today
    let sales_today: f64 = conn.query_row(
        "SELECT COALESCE(SUM(grand_total), 0) FROM orders WHERE date(order_date) = date('now', 'localtime')",
        [],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    // Sales Month
    let sales_month: f64 = conn.query_row(
        "SELECT COALESCE(SUM(grand_total), 0) FROM orders WHERE strftime('%Y-%m', order_date) = strftime('%Y-%m', 'now', 'localtime')",
        [],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    // Total Purchases
    let total_purchases: f64 = conn.query_row(
        "SELECT COALESCE(SUM(total_amount), 0) FROM purchases",
        [],
        |row| row.get(0),
    ).unwrap_or(0.0);
    
    // Total Cost of Goods Sold (COGS) from order_items
    let total_cogs: f64 = conn.query_row(
        "SELECT COALESCE(SUM(quantity * buying_price_snapshot), 0) FROM order_items",
        [],
        |row| row.get(0),
    ).unwrap_or(0.0);

    // Total Profit = Total Sales - Total COGS
    let total_profit = total_sales - total_cogs;
    
    // Low Stock Count
    let low_stock_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE stock_quantity <= 5 AND is_deleted = 0",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    // Order Count
    let order_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM orders",
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    // Product Count
    let product_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM products WHERE is_deleted = 0",
        [],
        |row| row.get(0),
    ).unwrap_or(0);
    
    Ok(DashboardStats {
        total_sales,
        sales_today,
        sales_month,
        total_purchases,
        total_profit,
        low_stock_count,
        order_count,
        product_count,
    })
}

#[tauri::command]
pub fn get_sales_report(start_date: String, end_date: String, db: State<Database>) -> Result<Vec<SalesReportItem>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    // Calculate profit per order: Grand Total - Sum(Item Cost)
    let mut stmt = conn.prepare(
        "SELECT 
            o.order_id, 
            o.order_date, 
            o.customer_name, 
            o.grand_total, 
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
            profit: row.get(4)?,
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
        "SELECT id, product_name, stock_quantity, unit, buying_price, (stock_quantity * buying_price) as stock_value 
         FROM products 
         WHERE is_deleted = 0 
         ORDER BY stock_quantity ASC"
    ).map_err(|e| e.to_string())?;
    
    let rows = stmt.query_map([], |row| {
        Ok(InventoryReportItem {
            id: row.get(0)?,
            name: row.get(1)?,
            stock: row.get(2)?,
            unit: row.get(3)?,
            cost_price: row.get(4)?,
            stock_value: row.get(5)?,
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


