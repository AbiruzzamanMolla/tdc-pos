# TDC-POS Calculation Documentation

This document outlines the mathematical formulas and logic used across all modules of the TDC-POS system, ensuring transparency in financial reporting and inventory management.

---

## 1. Product Management View (`Products.vue`)

### 1.1 Buying Cost (Internal Expense)

Difference between the current average buying price and the original price of the product.
**Formula:**
$$Buying\ Cost = Buying\ Price - Original\ Price$$

### 1.2 Expected Profit

Potential profit based on the difference between the selling price and the average buying price.
**Formula:**
$$Expected\ Profit = Default\ Selling\ Price - Buying\ Price$$

### 1.3 Expected Selling Price (Virtual Field)

A suggested selling price based on a user-defined profit margin.
**Formula:**
$$Expected\ Selling\ Price = Buying\ Price \times \left(1 + \frac{Profit\ \%}{100}\right)$$

---

## 2. Procurement / Buying Module (`Buying.vue`)

### 2.1 Line Item Subtotal

**Formula:**
$$Subtotal = (Quantity \times Unit\ Price) + Extra\ Charge$$

### 2.2 Landed Cost per Unit

The true cost of one unit, including distributed extra charges.
**Formula:**
$$Landed\ Cost = \frac{(Quantity \times Unit\ Price) + Extra\ Charge}{Quantity}$$

### 2.3 Weighted Average Costing (WAC) Logic

Updates the `Buying Price` in the product record after every purchase.
**Formula:**
$$New\ Average\ Price = \frac{(Old\ Qty \times Old\ Price) + (New\ Qty \times Landed\ Cost)}{Total\ Qty}$$

---

## 3. Point of Sale / Selling Module (`Selling.vue`)

### 3.1 Item Subtotal

**Formula:**
$$Item\ Subtotal = Quantity \times Selling\ Price$$

### 3.2 Order Subtotal

**Formula:**
$$Order\ Subtotal = \sum(Item\ Subtotals)$$

### 3.3 Grand Total

Final amount to be paid by the customer.
**Formula:**
$$Grand\ Total = (Order\ Subtotal + Extra\ Charge + Delivery\ Fee) - Discount$$

---

## 4. Dashboard & Financial Reporting

### 4.1 Cost of Goods Sold (COGS)

Calculated using the "Buying Price Snapshot" stored at the moment of sale.
**Formula:**
$$COGS = \sum(Quantity\ Sold \times Buying\ Price\ Snapshot)$$

### 4.2 Periodic & Total Profit

Calculated by subtracting corresponding COGS from the sales grand total for the specific time period (Today, Month, Year, or Lifetime).
**Formula:**
$$Profit_{Period} = Total\ Sales_{Period} - COGS_{Period}$$

- **Temporal Precision:** The system aggregates these metrics using the database `order_date` filtered by `date()`, `strftime('%Y-%m')`, and `strftime('%Y')` respectively.

### 4.3 Inventory Valuation

Current value of all stock held in the warehouse.
**Formula:**
$$Stock\ Value = \sum_{all\ products}(Stock\ Quantity \times Buying\ Price)$$

---

## 5. Summary Table of Variables

| Variable         | Description                                          | Source          | Module          |
| :--------------- | :--------------------------------------------------- | :-------------- | :-------------- |
| `Original Price` | The base price of the item from the source.          | User Input      | Products        |
| `Buying Price`   | The current Weighted Average Cost (WAC).             | Auto-Calculated | Procurement     |
| `Unit Price`     | Price per unit paid to supplier in a specific entry. | User Input      | Procurement     |
| `Extra Charge`   | Fixed additional cost (Shipping/Tax) for an entry.   | User Input      | Procurement/POS |
| `Landed Cost`    | Unit cost + distributed extra charges.               | Calculated      | Procurement     |
| `Snapshot`       | The WAC at the specific second a sale occurred.      | Database        | POS (Orders)    |
| `Profit %`       | User-defined margin for price suggestions.           | User Input      | Products        |
| `Tax %`          | Percentage added to subtotal (if applicable).        | User Input      | Products        |
