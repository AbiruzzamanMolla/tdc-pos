# TDC-POS

TDC-POS is a modern, offline-first Point of Sale (POS) desktop application built with Tauri v2, Vue 3, and Rust. It is designed to manage day-to-day business operations efficiently, including sales, purchases, inventory, and reporting.

![TDC-POS](https://via.placeholder.com/800x450?text=TDC+POS+Screenshot)

## Features

- **Dashboard**: Real-time business overview with key metrics (Sales Today, Monthly Sales, Total Profit).
- **Product Management**:
  - Full CRUD (Create, Read, Update, Delete) functionality.
  - **Multiple Images**: Support for adding multiple images per product.
  - Stock tracking and low-stock indicators.
- **Point of Sale (POS)**:
  - Interactive product grid and cart system.
  - Fast checkout with customer details and discount management.
- **Procurement (Buying)**:
  - Record supplier purchases with automatic stock updates.
  - **Weighted Average Costing**: Automatic calculation of product average cost.
  - **Landed Cost**: Support for extra charges (shipping, tax) per purchase item.
- **Reports**:
  - Sales history with profit analysis.
  - Inventory valuation reports.
  - PDF export functionality.
- **Tools**:
  - **Floating Calculator**: Persistent calculator accessible from any screen.
- **Database**:
  - **Offline-First**: Uses local SQLite database.
  - **Backup & Restore**: Securely backup and restore your business data.
- **Configuration**:
  - Customizable store settings.
  - **Currency**: default currency is BDT (৳).

## Tech Stack

- **Frontend**: Vue 3 (Composition API), Tailwind CSS, Pinia
- **Backend**: Rust, Tauri v2
- **Database**: SQLite (via `rusqlite`)

## Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v16+)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (for Windows)

### Development

1.  Clone the repository:

    ```bash
    git clone https://github.com/your-repo/tdc-pos.git
    cd tdc-pos
    ```

2.  Install dependencies:

    ```bash
    npm install
    ```

3.  Run in development mode:
    ```bash
    npm run tauri dev
    ```

### Build

To build the application for production:

```bash
npm run tauri build
```

The executable will be available in `src-tauri/target/release/bundle/nsis/`.

## Version History

See [CHANGELOG.md](./CHANGELOG.md) for detailed version history.

## License

MIT License.
