PROJECT TYPE: Software Requirements Specification (SRS)
TARGET AI: Antigravity
PROJECT NAME: TDC-POS
PLATFORM: Tauri v2 (Desktop Application)
FRONTEND FRAMEWORK: Vue 3
BACKEND: Rust (Tauri Core)
DATABASE: SQLite (Local, Offline)
APP TYPE: Offline POS Management System

=====================================================

1. # PROJECT OVERVIEW

Build a full-featured desktop Point of Sale (POS) application named "TDC-POS" using Tauri version 2 with Vue 3 as the frontend framework.

The application will be used to manage:

- Products
- Purchases (Buying)
- Sales (Orders)
- Customers
- Profit & Loss
- Reports
- Database Backup & Restore

The application must work fully offline and store all data locally using SQLite.

# ===================================================== 2. TECHNOLOGY STACK (MANDATORY)

Frontend:

- Vue 3 (Composition API)
- Vue Router
- Pinia (state management)
- Tailwind CSS or similar UI framework

Backend:

- Rust (Tauri v2)
- SQLite database
- SQL migrations

Other:

- Local PDF generation
- Local file system access for backup/restore
- Cross-platform support (Windows, Linux, macOS)

# ===================================================== 3. USER ROLE

Admin (default user):

- Full access to all features
- No authentication required initially
- Architecture must support adding authentication later

# ===================================================== 4. CORE MODULES & FEATURES

---

## 4.1 PRODUCT MANAGEMENT MODULE

Product Table Fields:

- id (primary key)
- product_name
- product_code (unique / SKU / barcode)
- category
- brand
- buying_price (real cost price)
- default_selling_price
- stock_quantity
- unit (pcs, kg, liter, etc.)
- tax_percentage (optional)
- created_at
- updated_at
- is_deleted (soft delete)

Product Images:

- Supports multiple images per product (optional)

Features:

- Add product
- Edit product
- Soft delete product
- View product list
- Search by name or product code
- Low stock warning
- Automatic stock updates after purchase/sale

---

## 4.2 PURCHASE (BUY) MODULE

Purchase Table:

- purchase_id
- supplier_name
- supplier_phone
- invoice_number
- purchase_date
- total_amount
- notes
- created_at

Purchase Items Table:

- id
- purchase_id
- product_id
- quantity
- buying_price
- subtotal

Purchase Logic:

- Purchasing increases product stock
- Buying price updates average cost (preferred)
- Purchase history must be preserved
- Purchase reports must be generated

---

## 4.3 SALES / ORDER MANAGEMENT MODULE

Order Types:

- Local
- Online

Customer Fields:

- customer_name
- phone_number
- address (optional for local)

Order Table:

- order_id
- order_date
- order_type (local / online)
- customer_name
- customer_phone
- customer_address
- subtotal
- extra_charge
- delivery_charge
- discount
- grand_total
- payment_method (cash / card / mobile)
- notes

Order Items Table:

- id
- order_id
- product_id
- quantity
- selling_price (can be different from default selling price)
- subtotal
- buying_price_snapshot

Order Logic:

- Selling reduces stock
- Allow per-order price override
- Calculate profit per product and per order
- Store full order history

---

## 4.4 PROFIT & LOSS MODULE

Profit Formula:
Profit = (selling_price × quantity) − (buying_price × quantity) − extra_charges

Required Calculations:

- Profit per product
- Profit per order
- Daily profit
- Monthly profit
- Yearly profit
- Loss detection (negative profit)

---

## 4.5 DASHBOARD MODULE

Dashboard must display:

- Total Sales (Today / Month / Year)
- Total Purchases
- Total Profit
- Total Loss
- Total Orders
- Total Products
- Low Stock Products
- Quick stats cards

---

## 4.6 REPORTING MODULE

Report Types:

- Daily report
- Monthly report
- Yearly report
- Date-to-date report

Report Categories:

- Sales report
- Purchase report
- Profit/Loss report
- Stock report

Report Features:

- Date range filters
- Order type filter
- PDF export
- Print-friendly layout

---

## 4.7 PDF GENERATION MODULE

PDF Requirements:

- Generated locally (offline)
- Business name: TDC-POS
- Report title
- Date range
- Table data
- Totals summary
- Clean professional layout

---

## 4.8 DATABASE BACKUP & RESTORE MODULE

Backup Features:

- Manual database backup button
- Automatic timestamped backup
- Backup format: SQLite (.db) file
- User can choose backup location
- Show success/failure messages

Restore Features:

- Restore database from selected backup file
- Warning before restore (data overwrite)
- App restart after restore (if required)
- Validation to ensure correct database file

Optional Enhancements:

- Scheduled auto-backup (daily / weekly)
- Keep last N backups
- Backup size display

---

## 4.9 SETTINGS MODULE

Settings Page:

- Business name
- Currency symbol (Default: BDT ৳)
- Default tax
- Backup location
- Auto-backup toggle
- Reset database (danger action)

# ===================================================== 5. DATABASE DESIGN REQUIREMENTS

Use SQLite with relational tables:

- products
- purchases
- purchase_items
- orders
- order_items
- customers
- settings
- backups_log (optional)

Use:

- Foreign keys
- Indexes
- Transactions
- Migrations

# ===================================================== 6. UI / UX REQUIREMENTS

- Sidebar navigation
- Pages:
  - Dashboard
  - Products
  - Purchases
  - Orders
  - Reports
  - Backup & Restore
  - Settings
- POS-friendly design
- Keyboard-friendly
- Responsive window resizing
- Large buttons and readable fonts

# ===================================================== 7. NON-FUNCTIONAL REQUIREMENTS

- Offline-first
- Fast startup
- No internet dependency
- Secure local storage
- Modular and scalable codebase
- Easy future extension

# ===================================================== 8. FUTURE FEATURES (DO NOT IMPLEMENT NOW)

- User authentication
- Role-based access
- Cloud sync
- Barcode scanner support
- Mobile companion app

# ===================================================== 9. EXPECTED OUTPUT FROM ANTIGRAVITY

Antigravity must:

- Generate a full Tauri v2 project
- Use Vue 3 frontend
- Implement Rust backend logic
- Create SQLite schema and migrations
- Implement CRUD operations
- Implement reports and PDF export
- Implement database backup & restore
- Provide clean folder structure
- Write readable, commented code

# ===================================================== 10. FINAL INSTRUCTION

Build "TDC-POS" exactly as specified above.
Do not skip any module.
Do not simplify business logic.
Treat this as a real-world production POS system.
