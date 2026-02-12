# Changelog

## [0.11.0] - 2024-05-24

### Added

- **Weighted Average Costing**: Implemented automatic inventory costing method for precise financial tracking.
- **Landed Cost Calculation**: New `Extra Charge` field in Procurement to include shipping, customs, or other additional costs.
- **Procurement Redesign**:
  - Removed manual `Buying Price` input; the system now calculates it based on purchase history.
  - Automatically calculates `Purchase Unit Cost` (Landed Cost) for every item.
  - Comprehensive history view with landed cost and extra charge breakdown.
- **Product Safety**: Disabled manual editing of `Buying Price` and `Stock Quantity` in the product manager to preserve audit integrity.

### Changed

- Updated `Buying` (Procurement) page UI with advanced analytics and calculations.
- Modified `ProductPurchaseHistory` and `StockHistory` to include landed cost data.
- Database schema updated to store financial metadata for purchase items.

## [0.10.0] - 2024-05-23

### Added

- Created `ProductDetailsModal` as a reusable component for consistent product information display.
- Integrated product details view into Buying and Selling pages via info icons on product cards.
- Enabled stock movement history across all product-related views.

### Changed

- Refactored `Products.vue` and `Stocks.vue` to use the new shared `ProductDetailsModal`.
- Improved UI consistency for product information modals.

## [0.9.0] - 2026-02-12

### Added

- **Product Visuals**: Show product images in Buying, Selling, and Stock List pages.
- **Cart Enhancements**: Added small item thumbnails to the Buying and Selling cart lists for better identification.

## [0.8.0] - 2026-02-12

### Added

- **Stock Movement History**: Added detailed In/Out history to the Product Details modal.
- **Unified Inventory Tracking**: Track both purchases (IN) and sales (OUT) in a single historical view.

## [0.7.0] - 2026-02-12

### Added

- **Stock Purchase History**: Added a "Details" view in the Stock List page to see all purchase records (Date, Supplier, Invoice, Qty, Price) for any specific product.
- **Improved Stock Visibility**: Added an "Actions" column to the available stock table.

## [0.6.0] - 2026-02-12

### Added

- **New Product Fields**: Added `Original Price`, `Facebook Link` (optional), and `Product Link` (optional) to product management.
- **Auto-Calculations**:
  - **Buying Cost**: Automatically calculated as `Buying Price - Original Price`.
  - **Expected Profit**: Automatically calculated as `Selling Price - Buying Price`.
- **Product Details View**: Comprehensive modal to view full product details, metrics, and clickable social/web links.

### Changed

- **Product Table**: Added columns for `Buying Cost` and `Profit` directly in the list view for quick reference.
- **Product Form**: Updated layout to include the new fields.

### Added

- **Expanded RBAC**: Implemented 7 distinct roles (`super_admin`, `admin`, `manager`, `buy_manager`, `sell_manager`, `report_checker`, `inspector`).
- **User Management Panel**: Admins and Super Admins can now create, view, and delete user accounts.
- **Granular Permissions**:
  - Restricts access to specific sidebar items and routes based on role capabilities.
  - Dashboard metrics and quick actions are now permissions-aware.
- **System Protection**: Added safeguards in database initialization and UI to prevent deletion of the primary admin account.

### Changed

- **Initial Setup**: The default user `admin` is now assigned the `super_admin` role.
- **UI Structure**: Sidebar now includes a "System" segment for User Management.

## [0.4.0] - 2026-02-11

### Added

- **User Authentication**: Secure sign-in system with a dedicated login page.
- **Role-Based Access (RBAC)**: Restriction of administrative features (Backup, Settings) to admin accounts.
- **Stock List Page**: A streamlined view of all products with quantities and stock status indicators.
- **Auto-Backup System**:
  - Scheduled backups (Daily/Weekly).
  - Retention policy to keep only the last N backups.
  - Automatic backup directory selection and size display.
  - Automatic backup check on application launch.
- **Branding**: Updated interface with TDC-POS logo and styling.

### Changed

- **Version Display**: Updated to v0.4.0-BETA.
- **Dashboard**: Added quick link to Stock List.
- **Sidebar**: Improved aesthetics and added user info display.

## [0.3.0] - 2026-02-11

### Added

- **Editable Selling Prices**: Per-item price override in the Selling cart; discount auto-calculated from price differences.
- **Base64 Image Display**: New `read_image_base64` backend command for reliable cross-platform image display.
- **Delete Functionality**: Delete buttons with stock reversal for both Buying entries and Sales.
- **Product Thumbnails**: Product list now shows image thumbnails.

### Changed

- **Renamed Pages**: "Stocks" → "Buying", "Orders" → "Selling" for clearer UX.
- **Buying Page Redesign**: Buying now uses POS-style two-panel layout (products + cart) matching the Selling page.
- **Responsive Tables**: All tables now scroll horizontally on small screens with min-width constraints.
- **Responsive Layouts**: All pages adapt to smaller window sizes with flex-col fallbacks.
- **Currency from Settings**: All pages dynamically use the currency symbol from Settings.

### Fixed

- **Image Loading**: Switched from `convertFileSrc` to base64 data URIs for reliable image display.
- **Dashboard Links**: Quick action links now point to correct Buying/Selling routes.
- **Version Display**: App sidebar shows correct version number.

## [0.2.1] - 2026-02-11

### Fixed

- **Images**: Fixed image persistence and loading by copying files to App Data directory.
- **Currency**: Fixed currency symbol display in Dashboard, Purchases, Orders, and Reports.
- **Details**: Added detailed view modals for Orders and Purchases.
- **Warnings**: Fixed unused code warnings in Rust backend.

## [0.2.0] - 2026-02-10

### Added

- **Product Images**: Products can now have multiple optional images.
- **Reporting Enhancements**: Improved sales and inventory reports.
- **Backup & Restore**: Secure database backup and restore functionality.
- **Settings**: Comprehensive settings for store information and configuration.

### Changed

- **Default Currency**: Default currency symbol changed to BDT (৳).
- **UI Improvements**: Updated product form to support image selection and display.
- **Documentation**: Updated README and SRS to reflect new features.

## [0.1.0] - 2026-02-09

### Added

- Initial release of TDC-POS.
- **Dashboard**: Real-time sales and stock metrics.
- **Product Management**: CRUD operations for products.
- **Purchases**: Purchase entry and stock management.
- **Orders (POS)**: Point of Sale interface, cart, and checkout.
- **Database**: SQLite integration with automatic migrations.
- **Offline Support**: Fully functional offline architecture.
