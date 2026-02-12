# Changelog

## [0.17.1] - 2026-02-12

### Fixed

- **Database Cleanup**: Fixed an issue where the "Clean Database" command was sending invalid arguments to the backend (camelCase vs snake_case mismatch), now effectively allowing Super Admins to wipe data.

## [0.17.0] - 2026-02-12

### Added

- **Collapsible Sidebar**: New toggle mechanism for the sidebar. On desktop, it can be collapsed to maximize workspace. On mobile/tablet, it behaves as a slide-out drawer with backdrop overlay.
- **Top Bar Navigation**: Added a persistent top bar (visible when sidebar is closed/mobile) containing the sidebar toggle and breadcrumb/page title.

### Changed

- **Responsive Design Overhaul**:
  - **Buying & Selling Views**: Updated split-view layouts (Catalog + Cart) to break into vertical stacks on smaller screens (up to XL/1280px width) for better usability on laptops and tablets.
  - **Dashboard**: Header layout now stacks vertically on mobile; stats cards adapt continuously from 1 to 4 columns.
  - **Table Layouts**: All data tables (Users, Purchases, Sales) are now wrapped in horizontal scrolling containers to prevent layout breakage on small screens.
  - **Modals**: Improved mobile spacing and max-width constraints.

## [0.16.0] - 2026-02-12

### Added

- **Database Cleanup System**: Super Admins can now selectively wipe Sales History, Purchase History, Activity Logs, or All Inventory from the Settings page. This action is protected by a random confirmation code.
- **Improved Dark Mode Support**: Additional global CSS overrides for `border-gray-*`, `::placeholder`, and alert colors (`bg-red-50`, `text-red-700`) to ensure better readability and consistency in dark mode.

### Fixed

- **Dark Theme Contrast**: Fixed issues where borders and specific background utilities (like in the cleanup or backup sections) were remaining light/white in dark mode.

## [0.15.0] - 2026-02-12

### Added

- **Global Theme Propagation**: All page-level elements (headings, cards, inputs, buttons, badges, table headers, focus rings) now respond to the active theme via CSS custom property overrides in `style.css`. Changing themes in the sidebar instantly updates every page's appearance.
- **Activity Log System**: Full audit trail for all system actions.
  - **Backend**: New `activity_logs` database table, `log_activity` and `get_activity_logs` Tauri commands.
  - **Frontend**: Reusable `logActivity()` helper (`utils/activityLogger.js`) auto-injects user context.
  - **ActivityLog.vue Page**: Paginated list with color-coded action badges, search, filter by action type and entity, relative timestamps ("2m ago"), and "Load More" pagination.
  - **Instrumented Pages**: Products (create/update/delete), Buying (create/delete), Selling (create/delete), Users (create/delete/password change/role change), Login (login/setup), Settings (save), Backup (backup/restore) — all emit activity logs automatically.
- **Dark Theme**: Added a full `Dark` theme option (🖤) with deep blacks and grays for low-light environments.
- **Activity Log Permission**: New `canViewActivityLog` computed property in auth store, granted to Super Admin, Admin, Manager, and Inspector roles.
- **Sidebar**: Activity Log nav link under System section. System section visibility updated to include `canViewActivityLog`.

### Changed

- **Theme Store**: Expanded with additional tokens: `mainTextSecondary`, `mainInputBg`, `mainInputBorder`, `mainTableHeaderBg`, `mainBadgeBg/Text`, `btnPrimary*`.
- **style.css**: Comprehensive global CSS overrides using `var(--t-*)` custom properties.

## [0.14.0] - 2026-02-12

### Added

- **Centralized Theme System**: Introduced a Pinia-based theme engine (`stores/theme.js`) with 6 modern color themes:
  - 🌙 **Midnight** — Classic dark sidebar with blue accents (default)
  - 🌊 **Ocean** — Deep navy with cyan highlights
  - 🌿 **Emerald** — Rich forest green with emerald accents
  - 🌅 **Sunset** — Warm stone with orange accents
  - 👑 **Royal** — Deep indigo with purple accents
  - 🌹 **Rose** — Dark navy with rose-pink accents
- **Theme Picker in Sidebar**: A "🎨 Theme" button in the sidebar footer toggles a compact 3×2 grid of theme swatches. Selecting a theme instantly applies it and persists the choice in localStorage.
- **CSS Custom Properties**: All theme colors are injected as CSS custom variables (`--t-*`), enabling any component to reference the active theme tokens.

### Changed

- **App.vue**: Completely rebuilt to use CSS classes driven by custom properties instead of hardcoded Tailwind colors. Sidebar, nav links, section headers, logout button, and footer all respond to theme changes in real-time.

## [0.13.0] - 2026-02-12

### Added

- **Secure First-Install Setup**: On first launch (when no users exist), the app shows a dedicated setup screen to create a Super Admin account with username, password, and confirmation. No default credentials are ever exposed.
- **Password Management**:
  - Any logged-in user can change their own password (requires current password verification).
  - Super Admins can reset any user's password without knowing the current one.
- **Role Management**: Super Admins can change any user's role directly from the User Management page via a dedicated Role modal.
- **Centralized Version Number**: Introduced `src/version.js` as the single source of truth for the app version. The sidebar footer and login page both reference this file.
- **Backend Commands**: Added `check_setup_required`, `setup_admin`, `change_password`, and `update_user_role` Tauri commands.

### Changed

- **Login Page**: Removed hardcoded "Default: admin / admin123" text. Login page now conditionally shows setup mode or sign-in mode depending on database state.
- **User Management Page**: Redesigned with action buttons for Password reset, Role change, and Delete. Status toast notifications replace `alert()` calls.
- **Sidebar Version**: Now dynamically reads from `version.js` instead of a hardcoded string.

### Security

- **Removed Default Admin Seed**: The database initialization (`db.rs`) no longer inserts a default admin user. All accounts must be created through the secure setup flow.
- **Password Validation**: Minimum 6-character password requirement enforced on setup, user creation, and password changes.

## [0.12.0] - 2026-02-12

### Added

- **Profit Percentage & Expected Selling Price**: Products now support a configurable profit margin (`profit_percentage`) with a real-time computed `Expected Selling Price`.
- **Executive Dashboard Redesign**:
  - Full Daily / Monthly / Yearly metrics for **Sales**, **Purchases**, and **Profits**.
  - Tabbed data visualization (Sales, Purchases, Profit tabs).
  - Live **Inventory Valuation** (Stock Qty × Avg Cost).
  - Financial KPIs with snapshot-based COGS accuracy.
- **Bangla / English Language Switching**: Dashboard supports runtime language toggle with persistent preference via `i18n` store.
- **Reports & Analytics Overhaul**:
  - 6 summary KPI cards for Sales reports (Revenue, Net Profit, Discounts, Orders, Avg Order Value, Profit Margin).
  - 5 summary KPI cards for Inventory reports (Cost Value, Retail Value, Potential Profit, Out of Stock, Low Stock).
  - Date preset buttons (Today, Week, Month, Year, All).
  - Live search filtering across all report data.
  - Enhanced PDF export with footer totals and margin data.
  - Inventory report now includes Category, Selling Price, and per-item Margin %.
- **Product Form Reordering**: Price fields in Add/Edit product now follow a logical flow: Original Price → Buying Price → Profit % → Expected Selling Price → Final Selling Price.
- **CALCULATIONS.md**: Comprehensive documentation covering all system formulas across Products, Procurement, POS, and Dashboard modules.

### Changed

- **Backup & Restore Page**: Polished UI with status toasts, better empty states, improved restore workflow, and best-practice tips.
- **Product Details Modal**: Now displays Profit % badge and Expected Selling Price metric.
- **Dashboard**: Replaced flat metric cards with premium gradient KPI cards and dark-mode system integrity panel.

### Fixed

- **Dashboard Profit Accuracy**: Profit calculations now use `buying_price_snapshot` from `order_items` joined with `orders` for precise temporal filtering.

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
