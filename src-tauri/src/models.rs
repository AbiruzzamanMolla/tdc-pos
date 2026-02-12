use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: Option<i64>,
    pub product_name: String,
    pub product_code: Option<String>,
    pub category: Option<String>,
    pub brand: Option<String>,
    pub buying_price: f64,
    pub default_selling_price: f64,
    pub stock_quantity: f64,
    pub unit: Option<String>,
    pub tax_percentage: f64,
    pub original_price: f64,
    pub profit_percentage: f64,
    pub facebook_link: Option<String>,
    pub product_link: Option<String>,
    pub created_at: Option<String>,

    pub updated_at: Option<String>,
    pub is_deleted: i32,
    pub images: Option<Vec<String>>, // Not a DB column, populated manually
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ProductImage {
    pub id: Option<i64>,
    pub product_id: i64,
    pub image_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Purchase {
    pub purchase_id: Option<i64>,
    pub supplier_name: Option<String>,
    pub supplier_phone: Option<String>,
    pub invoice_number: Option<String>,
    pub purchase_date: Option<String>,
    pub total_amount: f64,
    pub notes: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseItem {
    pub id: Option<i64>,
    pub purchase_id: Option<i64>,
    pub product_id: i64,
    pub quantity: f64,
    pub buying_price: f64, // acts as unit_price
    pub extra_charge: f64,
    pub subtotal: f64,      // acts as total_cost (qty * unit_price + extra_charge)
    pub purchase_unit_cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseItemDetail {
    pub id: Option<i64>,
    pub purchase_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: f64,
    pub buying_price: f64,
    pub extra_charge: f64,
    pub subtotal: f64,
    pub purchase_unit_cost: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub order_id: Option<i64>,
    pub order_date: Option<String>,
    pub order_type: String,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub customer_address: Option<String>,
    pub subtotal: f64,
    pub extra_charge: f64,
    pub delivery_charge: f64,
    pub discount: f64,
    pub grand_total: f64,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItem {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub quantity: f64,
    pub selling_price: f64,
    pub subtotal: f64,
    pub buying_price_snapshot: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemDetail {
    pub id: Option<i64>,
    pub order_id: Option<i64>,
    pub product_id: i64,
    pub product_name: String,
    pub quantity: f64,
    pub selling_price: f64,
    pub subtotal: f64,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    // Sales
    pub sales_today: f64,
    pub sales_month: f64,
    pub sales_year: f64,
    pub total_sales: f64,
    
    // Purchases
    pub purchases_today: f64,
    pub purchases_month: f64,
    pub purchases_year: f64,
    pub total_purchases: f64,
    
    // Profit
    pub profit_today: f64,
    pub profit_month: f64,
    pub profit_year: f64,
    pub total_profit: f64,
    
    // Inventory & General
    pub inventory_value: f64,
    pub low_stock_count: i64,
    pub order_count: i64,
    pub product_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesReportItem {
    pub order_id: i64,
    pub date: String,
    pub customer: Option<String>,
    pub total: f64,
    pub discount: f64,
    pub items_count: i64,
    pub profit: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryReportItem {
    pub id: i64,
    pub name: String,
    pub category: Option<String>,
    pub stock: f64,
    pub unit: Option<String>,
    pub cost_price: f64,
    pub selling_price: f64,
    pub stock_value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub password: Option<String>,
    pub role: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created_at: String,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductPurchaseHistory {
    pub date: String,
    pub supplier_name: Option<String>,
    pub invoice_number: Option<String>,
    pub quantity: f64,
    pub buying_price: f64,
    pub extra_charge: f64,
    pub subtotal: f64,
    pub purchase_unit_cost: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct StockMovement {
    pub date: String,
    pub movement_type: String, // "IN" or "OUT"
    pub entity_name: Option<String>, // Supplier or Customer
    pub reference: Option<String>, // Invoice or Order ID
    pub quantity: f64,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Option<i64>,
    pub user_id: Option<i64>,
    pub username: String,
    pub action: String,        // CREATE, UPDATE, DELETE, LOGIN, BACKUP, RESTORE, etc.
    pub entity_type: String,   // Product, Order, Purchase, User, Settings, Backup
    pub entity_id: Option<i64>,
    pub description: String,
    pub created_at: Option<String>,
}
