# Changelog

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
