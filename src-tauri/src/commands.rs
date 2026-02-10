use crate::models::{Product, Purchase, PurchaseItem, Order, OrderItem};
use crate::db::Database;
use tauri::State;
use rusqlite::params;

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
        })
    }).map_err(|e| e.to_string())?;
    
    let mut products = Vec::new();
    for product in products_iter {
        products.push(product.map_err(|e| e.to_string())?);
    }
    
    Ok(products)
}

#[tauri::command]
pub fn create_product(product: Product, db: State<Database>) -> Result<i64, String> {
   let conn = db.conn.lock().map_err(|e| e.to_string())?;
   
   conn.execute(
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
   
   Ok(conn.last_insert_rowid())
}

#[tauri::command]
pub fn update_product(product: Product, db: State<Database>) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    
    conn.execute(
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
