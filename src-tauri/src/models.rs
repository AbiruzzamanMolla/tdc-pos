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
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub is_deleted: i32,
    pub images: Option<Vec<String>>, // Not a DB column, populated manually
}

#[derive(Debug, Serialize, Deserialize)]
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
    pub buying_price: f64,
    pub subtotal: f64,
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
pub struct DashboardStats {
    pub total_sales: f64,
    pub sales_today: f64,
    pub sales_month: f64,
    pub total_purchases: f64,
    pub total_profit: f64,
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
    pub profit: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InventoryReportItem {
    pub id: i64,
    pub name: String,
    pub stock: f64,
    pub unit: Option<String>,
    pub cost_price: f64,
    pub stock_value: f64,
}
