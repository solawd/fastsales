use axum::{
    Json,
    extract::{Path, State, Extension, Query},
    http::StatusCode,
};


use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use serde::Deserialize;
use utoipa::IntoParams;

use jsonwebtoken::{EncodingKey, Header};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use rand_core::OsRng;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;
use chrono::{Utc, Datelike};

use crate::AppState;
use crate::auth::Claims;
use shared::models::{
    Customer, CustomerInput, CustomerDetails, Product, ProductDetails, ProductInput, ProductType,
    SaleItem, SaleItemInput, Staff, StaffInput, UploadResponse, SalesStats, DailySales, SalesItemsListResponse,
    TopProduct, Sale, SaleInput, ProductSalesSummary,
};

#[derive(Deserialize)]
pub struct SalesSearchParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub query: Option<String>,
}



#[derive(Deserialize)]
pub struct SearchParams {
    pub search: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List all products with optional search and pagination", body = [Product]))
)]
pub async fn list_products(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Product>>, StatusCode> {
    let rows = if let Some(search) = params.search {
        let pattern = format!("%{}%", search);
        sqlx::query(
            "SELECT id, name, description, price_cents, stock, product_type 
             FROM products 
             WHERE name LIKE ? 
                OR description LIKE ? 
                OR EXISTS (
                    SELECT 1 FROM product_details 
                    WHERE product_details.product_id = products.id 
                    AND (detail_name LIKE ? OR detail_value LIKE ?)
                )"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?
    } else {
        sqlx::query("SELECT id, name, description, price_cents, stock, product_type FROM products")
            .fetch_all(&state.db)
            .await
            .map_err(map_db_err)?
    };

    let mut products = Vec::with_capacity(rows.len());
    for row in rows {
        let mut product = product_from_row(&row)?;
        let details_rows = sqlx::query(
            "SELECT product_id, detail_name, detail_value FROM product_details WHERE product_id = ?",
        )
        .bind(product.id.to_string())
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;

        let details: Vec<ProductDetails> = details_rows
            .into_iter()
            .map(|row| ProductDetails {
                product_id: parse_uuid(row.get("product_id")).unwrap_or_default(),
                detail_name: row.get("detail_name"),
                detail_value: row.get("detail_value"),
            })
            .collect();
        
        product.details = details;
        products.push(product);
    }
    Ok(Json(products))
}

#[utoipa::path(
    post,
    path = "/api/products",
    tag = "Products",
    request_body = ProductInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Create a new product with details", body = Product))
)]
pub async fn create_product(
    State(state): State<AppState>,
    Json(input): Json<ProductInput>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    let product_id = Uuid::new_v4();
    let product = Product {
        id: product_id,
        name: input.name,
        description: input.description,
        price_cents: input.price_cents,
        stock: input.stock,
        product_type: input.product_type,

        details: input.details.iter().map(|d| ProductDetails {
            product_id,
            detail_name: d.detail_name.clone(),
            detail_value: d.detail_value.clone(),
        }).collect(),
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    sqlx::query(
        "INSERT INTO products (id, name, description, price_cents, stock, product_type) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(product.id.to_string())
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price_cents)
    .bind(product.stock)
    .bind(product.product_type.as_str())
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    for detail in input.details {
        sqlx::query(
            "INSERT INTO product_details (id, product_id, detail_name, detail_value) VALUES (?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(product_id.to_string())
        .bind(&detail.detail_name)
        .bind(&detail.detail_value)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(product)))
}

#[utoipa::path(
    get,
    path = "/api/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve a specific product by its unique ID", body = Product), (status = 404))
)]
pub async fn get_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Product>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, name, description, price_cents, stock, product_type FROM products WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let mut product = product_from_row(&row)?;
    let details_rows = sqlx::query(
        "SELECT product_id, detail_name, detail_value FROM product_details WHERE product_id = ?",
    )
    .bind(product.id.to_string())
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    product.details = details_rows
        .into_iter()
        .map(|row| ProductDetails {
            product_id: parse_uuid(row.get("product_id")).unwrap_or_default(),
            detail_name: row.get("detail_name"),
            detail_value: row.get("detail_value"),
        })
        .collect();

    Ok(Json(product))
}

#[utoipa::path(
    put,
    path = "/api/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    request_body = ProductInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Update an existing product's information", body = Product), (status = 404))
)]
pub async fn update_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<ProductInput>,
) -> Result<Json<Product>, StatusCode> {
    let product = Product {
        id,
        name: input.name,
        description: input.description,
        price_cents: input.price_cents,
        stock: input.stock,
        product_type: input.product_type,
        details: input.details.iter().map(|d| ProductDetails {
            product_id: id,
            detail_name: d.detail_name.clone(),
            detail_value: d.detail_value.clone(),
        }).collect(),
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    let result = sqlx::query(
        "UPDATE products SET name = ?, description = ?, price_cents = ?, stock = ?, product_type = ? WHERE id = ?",
    )
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price_cents)
    .bind(product.stock)
    .bind(product.product_type.as_str())
    .bind(product.id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Replace details
    sqlx::query("DELETE FROM product_details WHERE product_id = ?")
        .bind(product.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;

    for detail in input.details {
        sqlx::query(
            "INSERT INTO product_details (id, product_id, detail_name, detail_value) VALUES (?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(product.id.to_string())
        .bind(&detail.detail_name)
        .bind(&detail.detail_value)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok(Json(product))
}

#[utoipa::path(
    delete,
    path = "/api/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Permanently remove a product from the system"), (status = 404))
)]
pub async fn delete_product(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM products WHERE id = ?")
        .bind(id.to_string())
        .execute(&state.db)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

#[utoipa::path(
    get,
    path = "/api/customers",
    tag = "Customers",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List all customers with optional search and pagination", body = [Customer]))
)]
pub async fn list_customers(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<Customer>>, StatusCode> {
    let rows = if let Some(search) = params.search {
        let pattern = format!("%{}%", search);
        sqlx::query(
            "SELECT id, first_name, last_name, middle_name, mobile_number, date_of_birth, email \
             FROM customers \
             WHERE first_name LIKE ? \
                OR last_name LIKE ? \
                OR middle_name LIKE ? \
                OR EXISTS (
                    SELECT 1 FROM customer_details \
                    WHERE customer_details.customer_id = customers.id \
                    AND detail_value LIKE ? \
                )"
        )
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .bind(&pattern)
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?
    } else {
        sqlx::query(
            "SELECT id, first_name, last_name, middle_name, mobile_number, date_of_birth, email FROM customers",
        )
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?
    };

    let mut customers = Vec::with_capacity(rows.len());
    for row in rows {
        let mut customer = customer_from_row(&row)?;
        let details_rows = sqlx::query(
            "SELECT customer_id, detail_name, detail_value FROM customer_details WHERE customer_id = ?",
        )
        .bind(customer.id.to_string())
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;

        let details: Vec<CustomerDetails> = details_rows
            .into_iter()
            .map(|row| CustomerDetails {
                customer_id: parse_uuid(row.get("customer_id")).unwrap_or_default(),
                detail_name: row.get("detail_name"),
                detail_value: row.get("detail_value"),
            })
            .collect();
        
        customer.details = details;
        customers.push(customer);
    }
    Ok(Json(customers))
}

#[utoipa::path(
    post,
    path = "/api/customers",
    tag = "Customers",
    request_body = CustomerInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Register a new customer", body = Customer), (status = 400))
)]
pub async fn create_customer(
    State(state): State<AppState>,
    Json(input): Json<CustomerInput>,
) -> Result<(StatusCode, Json<Customer>), StatusCode> {
    let customer_id = Uuid::new_v4();
    let customer = Customer {
        id: customer_id,
        first_name: input.first_name,
        last_name: input.last_name,
        middle_name: input.middle_name,
        mobile_number: input.mobile_number,
        date_of_birth: input.date_of_birth,
        email: input.email,
        details: input.details.iter().map(|d| CustomerDetails {
            customer_id,
            detail_name: d.detail_name.clone(),
            detail_value: d.detail_value.clone(),
        }).collect(),
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    sqlx::query(
        "INSERT INTO customers (id, first_name, last_name, middle_name, mobile_number, date_of_birth, email) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(customer.id.to_string())
    .bind(&customer.first_name)
    .bind(&customer.last_name)
    .bind(&customer.middle_name)
    .bind(&customer.mobile_number)
    .bind(&customer.date_of_birth)
    .bind(&customer.email)
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    for detail in input.details {
        sqlx::query(
            "INSERT INTO customer_details (id, customer_id, detail_name, detail_value) VALUES (?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(customer_id.to_string())
        .bind(&detail.detail_name)
        .bind(&detail.detail_value)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(customer)))
}

#[utoipa::path(
    get,
    path = "/api/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve a specific customer's profile", body = Customer), (status = 404))
)]
pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Customer>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, first_name, last_name, middle_name, mobile_number, date_of_birth, email FROM customers WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::NOT_FOUND)?;

    let mut customer = customer_from_row(&row)?;
    let details_rows = sqlx::query(
        "SELECT customer_id, detail_name, detail_value FROM customer_details WHERE customer_id = ?",
    )
    .bind(customer.id.to_string())
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    customer.details = details_rows
        .into_iter()
        .map(|row| CustomerDetails {
            customer_id: parse_uuid(row.get("customer_id")).unwrap_or_default(),
            detail_name: row.get("detail_name"),
            detail_value: row.get("detail_value"),
        })
        .collect();

    Ok(Json(customer))
}

#[utoipa::path(
    put,
    path = "/api/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    request_body = CustomerInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Update a customer's profile information", body = Customer), (status = 400), (status = 404))
)]
pub async fn update_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CustomerInput>,
) -> Result<Json<Customer>, StatusCode> {


    let customer = Customer {
        id,
        first_name: input.first_name,
        last_name: input.last_name,
        middle_name: input.middle_name,
        mobile_number: input.mobile_number,
        date_of_birth: input.date_of_birth,
        email: input.email,
        details: input.details.iter().map(|d| CustomerDetails {
            customer_id: id,
            detail_name: d.detail_name.clone(),
            detail_value: d.detail_value.clone(),
        }).collect(),
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    let result = sqlx::query(
        "UPDATE customers SET first_name = ?, last_name = ?, middle_name = ?, mobile_number = ?, date_of_birth = ?, email = ? WHERE id = ?",
    )
    .bind(&customer.first_name)
    .bind(&customer.last_name)
    .bind(&customer.middle_name)
    .bind(&customer.mobile_number)
    .bind(&customer.date_of_birth)
    .bind(&customer.email)
    .bind(customer.id.to_string())
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Replace details
    sqlx::query("DELETE FROM customer_details WHERE customer_id = ?")
        .bind(customer.id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;

    for detail in input.details {
        sqlx::query(
            "INSERT INTO customer_details (id, customer_id, detail_name, detail_value) VALUES (?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(customer.id.to_string())
        .bind(&detail.detail_name)
        .bind(&detail.detail_value)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok(Json(customer))
}

#[utoipa::path(
    delete,
    path = "/api/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Permanently remove a customer"), (status = 404))
)]
pub async fn delete_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM customers WHERE id = ?")
        .bind(id.to_string())
        .execute(&state.db)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

#[utoipa::path(
    get,
    path = "/api/sales",
    tag = "Sales",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List individual sale items with filtering by date and pagination", body = SalesItemsListResponse))
)]
pub async fn list_sales(
    State(state): State<AppState>,
    Query(params): Query<SalesSearchParams>,
) -> Result<Json<SalesItemsListResponse>, StatusCode> {
    let mut query = "SELECT id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note FROM sale_items".to_string();
    let mut args = Vec::new();

    let (start_date, end_date) = get_default_dates(params.start_date.clone(), params.end_date.clone());

    query.push_str(" WHERE date(date_of_sale) >= date(?) AND date(date_of_sale) <= date(?)");
    args.push(start_date);
    args.push(end_date);
    
    // Sort by date descending
    query.push_str(" ORDER BY date_of_sale DESC");

    // Pagination
    let limit = params.limit.unwrap_or(20);
    // page defaults to 1
    let page = params.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    query.push_str(" LIMIT ? OFFSET ?");
    
    let mut sql_query = sqlx::query(&query);
    for arg in &args {
        sql_query = sql_query.bind(arg);
    }
    sql_query = sql_query.bind(limit).bind(offset);

    let rows = sql_query
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut sales = Vec::with_capacity(rows.len());
    for row in rows {
        sales.push(sale_item_from_row(&row)?);
    }
    
    // Calculate total for the period (DB side query usually better, but for small datasets iterating is fine too. User asked for DB query)
    // Let's run a separate COUNT/SUM query as requested.
    let sum_query = "SELECT SUM(total_cents) as total FROM sale_items WHERE date(date_of_sale) >= date(?) AND date(date_of_sale) <= date(?)".to_string();
    
    let mut sql_sum_query = sqlx::query(&sum_query);
    for arg in &args {
        sql_sum_query = sql_sum_query.bind(arg);
    }
    
    let row = sql_sum_query.fetch_one(&state.db).await.map_err(map_db_err)?;
    let total_sales_period_cents: i64 = row.try_get("total").unwrap_or(0);

    Ok(Json(SalesItemsListResponse {
        sales,
        total_sales_period_cents,
    }))
}

#[utoipa::path(
    post,
    path = "/api/sales",
    tag = "Sales",
    request_body = SaleItemInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Record a single sale item (for legacy or single-item sales)", body = SaleItem))
)]
pub async fn create_sale(
    State(state): State<AppState>,
    Json(input): Json<SaleItemInput>,
) -> Result<(StatusCode, Json<SaleItem>), StatusCode> {
    let sale = SaleItem {
        id: Uuid::new_v4(),
        sale_id: input.sale_id,
        product_id: input.product_id,
        customer_id: input.customer_id,
        date_of_sale: input.date_of_sale,
        quantity: input.quantity,
        discount: input.discount,
        total_cents: input.total_cents,
        total_resolved: input.total_resolved,
        note: input.note,
        product_name: None,
        price_per_item: None,
    };

    sqlx::query(
        "INSERT INTO sale_items (id, sale_id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note, product_name, price_per_item) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, (SELECT name FROM products WHERE id = ?), (SELECT price_cents FROM products WHERE id = ?))",
    )
    .bind(sale.id.to_string())
    .bind(sale.sale_id.map(|id| id.to_string()))
    .bind(sale.product_id.to_string())
    .bind(sale.customer_id.map(|id| id.to_string()))
    .bind(&sale.date_of_sale)
    .bind(sale.quantity)
    .bind(sale.discount)
    .bind(sale.total_cents)
    .bind(sale.total_resolved)
    .bind(&sale.note)
    .bind(sale.product_id.to_string())
    .bind(sale.product_id.to_string())
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    // Fetch the created item to get product_name and price_per_item and ensure consistency
    let row = sqlx::query("SELECT sale_items.id, sale_items.sale_id, sale_items.product_id, sale_items.customer_id, sale_items.date_of_sale, sale_items.quantity, sale_items.discount, sale_items.total_cents, sale_items.total_resolved, sale_items.note, COALESCE(sale_items.product_name, products.name) as product_name, COALESCE(sale_items.price_per_item, products.price_cents) as price_per_item FROM sale_items LEFT JOIN products ON sale_items.product_id = products.id WHERE sale_items.id = ?")
        .bind(sale.id.to_string())
        .fetch_one(&state.db)
        .await
        .map_err(map_db_err)?;
        
    let sale = sale_item_from_row(&row)?;

    Ok((StatusCode::CREATED, Json(sale)))
}

#[utoipa::path(
    post,
    path = "/api/sales_transactions",
    tag = "Sales",
    request_body = SaleInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Create a new sales transaction containing multiple items", body = Sale))
)]
pub async fn create_sales_transaction(
    State(state): State<AppState>,
    Json(input): Json<shared::models::SaleInput>,
) -> Result<(StatusCode, Json<shared::models::Sale>), StatusCode> {
    let sale_id = Uuid::new_v4();
    let mut sale = shared::models::Sale {
        id: sale_id,
        customer_id: input.customer_id,
        date_and_time: input.date_and_time,
        sale_items: input.sale_items.iter().map(|item_input| shared::models::SaleItem {
            id: Uuid::new_v4(),
            sale_id: Some(sale_id),
            product_id: item_input.product_id,
            customer_id: item_input.customer_id,
            date_of_sale: item_input.date_of_sale,
            quantity: item_input.quantity,
            discount: item_input.discount,
            total_cents: item_input.total_cents,
            total_resolved: item_input.total_resolved,
            note: item_input.note.clone(),
            product_name: None,
            price_per_item: None,
        }).collect(),
        total_cents: input.total_cents,
        discount: input.discount,
        total_resolved: input.total_resolved,
        sales_channel: input.sales_channel,
        staff_responsible: input.staff_responsible,
        company_branch: input.company_branch,
        car_number: input.car_number,
        receipt_number: input.receipt_number,
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    sqlx::query(
        "INSERT INTO sales (id, customer_id, date_and_time, total_cents, discount, total_resolved, sales_channel, staff_responsible, company_branch, car_number, receipt_number) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(sale.id.to_string())
    .bind(sale.customer_id.map(|id| id.to_string()))
    .bind(&sale.date_and_time)
    .bind(sale.total_cents)
    .bind(sale.discount)
    .bind(sale.total_resolved)
    .bind(sale.sales_channel.to_string())
    .bind(sale.staff_responsible.to_string())
    .bind(&sale.company_branch)
    .bind(&sale.car_number)
    .bind(&sale.receipt_number)
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    for item in &sale.sale_items {
        sqlx::query(
            "INSERT INTO sale_items (id, sale_id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note, product_name, price_per_item) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, (SELECT name FROM products WHERE id = ?), (SELECT price_cents FROM products WHERE id = ?))",
        )
        .bind(item.id.to_string())
        .bind(item.sale_id.map(|id| id.to_string()))
        .bind(item.product_id.to_string())
        .bind(item.customer_id.map(|id| id.to_string()))
        .bind(&item.date_of_sale)
        .bind(item.quantity)
        .bind(item.discount)
        .bind(item.total_cents)
        .bind(item.total_resolved)
        .bind(&item.note)
        .bind(item.product_id.to_string())
        .bind(item.product_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    // Re-fetch items to get product_names and price_per_item
    let items_rows = sqlx::query(
        "SELECT sale_items.id, sale_items.sale_id, sale_items.product_id, sale_items.customer_id, sale_items.date_of_sale, sale_items.quantity, sale_items.discount, sale_items.total_cents, sale_items.total_resolved, sale_items.note, COALESCE(sale_items.product_name, products.name) as product_name, COALESCE(sale_items.price_per_item, products.price_cents) as price_per_item 
         FROM sale_items 
         LEFT JOIN products ON sale_items.product_id = products.id 
         WHERE sale_id = ?"
    )
    .bind(sale.id.to_string())
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_err)?;

    let items = items_rows.into_iter().map(|r| sale_item_from_row(&r)).collect::<Result<Vec<_>, _>>()?;
    sale.sale_items = items;

    tx.commit().await.map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(sale)))
}


#[utoipa::path(
    get,
    path = "/api/sales_transactions",
    tag = "Sales",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List sales transactions with filtering", body = [Sale]))
)]
pub async fn list_sales_transactions(
    State(state): State<AppState>,
    Query(params): Query<SalesSearchParams>,
) -> Result<Json<Vec<Sale>>, StatusCode> {
    let mut query = "SELECT sales.* FROM sales LEFT JOIN customers ON sales.customer_id = customers.id".to_string();
    let mut conditions = Vec::new();
    let mut args = Vec::new();

    if let Some(q) = &params.query {
        if !q.is_empty() {
            let pattern = format!("%{}%", q);
            conditions.push("(customers.first_name LIKE ? OR customers.last_name LIKE ? OR sales.receipt_number LIKE ?)");
            args.push(pattern.clone());
            args.push(pattern.clone());
            args.push(pattern.clone());
        }
    }

    if let Some(start) = &params.start_date {
        if !start.is_empty() {
             conditions.push("sales.date_and_time >= ?");
             args.push(start.clone());
        }
    }
    
    if let Some(end) = &params.end_date {
        if !end.is_empty() {
             conditions.push("sales.date_and_time <= ?");
             args.push(end.clone());
        }
    }

    if !conditions.is_empty() {
        query.push_str(" WHERE ");
        query.push_str(&conditions.join(" AND "));
    }
    
    query.push_str(" ORDER BY sales.date_and_time DESC");

    // Pagination
    let limit = params.limit.unwrap_or(20);
    let page = params.page.unwrap_or(1);
    let offset = (page - 1) * limit;

    query.push_str(" LIMIT ? OFFSET ?");
    
    let mut sql_query = sqlx::query(&query);
    for arg in &args {
        sql_query = sql_query.bind(arg);
    }
    sql_query = sql_query.bind(limit).bind(offset);

    let rows = sql_query
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut sales = Vec::with_capacity(rows.len());
    for row in rows {
        sales.push(sale_from_row(&row)?);
    }

    Ok(Json(sales))
}

#[utoipa::path(
    get,
    path = "/api/sales_transactions/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale UUID")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve a full sales transaction", body = Sale), (status = 404))
)]
pub async fn get_sales_transaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Sale>, StatusCode> {
    let row = sqlx::query("SELECT * FROM sales WHERE id = ?")
        .bind(id.to_string())
        .fetch_optional(&state.db)
        .await
        .map_err(map_db_err)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let mut sale = sale_from_row(&row)?;

    // Fetch items
    let items_rows = sqlx::query(
        "SELECT sale_items.id, sale_items.sale_id, sale_items.product_id, sale_items.customer_id, sale_items.date_of_sale, sale_items.quantity, sale_items.discount, sale_items.total_cents, sale_items.total_resolved, sale_items.note, COALESCE(sale_items.product_name, products.name) as product_name, COALESCE(sale_items.price_per_item, products.price_cents) as price_per_item 
         FROM sale_items 
         LEFT JOIN products ON sale_items.product_id = products.id 
         WHERE sale_id = ?"
    )
    .bind(sale.id.to_string())
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let items = items_rows.into_iter().map(|r| sale_item_from_row(&r)).collect::<Result<Vec<_>, _>>()?;
    sale.sale_items = items;

    Ok(Json(sale))
}

#[utoipa::path(
    get,
    path = "/api/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve a specific sale item", body = SaleItem), (status = 404))
)]
pub async fn get_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<SaleItem>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note FROM sale_items WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(sale_item_from_row(&row)?))
}

#[utoipa::path(
    get,
    path = "/api/sales/stats/today",
    tag = "Reports",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get sales statistics for the current day", body = SalesStats))
)]
pub async fn get_today_sales(
    State(state): State<AppState>,
) -> Result<Json<shared::models::SalesStats>, StatusCode> {
    let row = sqlx::query(
        "SELECT SUM(total_resolved) as total, COUNT(*) as count FROM sale_items WHERE date(date_of_sale) = date('now')",
    )
    .fetch_one(&state.db)
    .await
    .map_err(map_db_err)?;

    let total: Option<i64> = row.try_get("total").unwrap_or(Some(0));
    let count: i64 = row.try_get("count").unwrap_or(0);

    Ok(Json(shared::models::SalesStats {
        total_sales_cents: total.unwrap_or(0),
        count,
    }))
}

#[utoipa::path(
    get,
    path = "/api/sales/stats/week",
    tag = "Reports",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get daily sales statistics for the current week (Mon-Sun)", body = [DailySales]))
)]
pub async fn get_weekly_sales_stats(
    State(state): State<AppState>,
) -> Result<Json<Vec<shared::models::DailySales>>, StatusCode> {
    // Logic: Find the most recent Monday (or today if it's Monday) and query from there.
    // In SQLite, DATE('now', 'weekday 1') returns the *next* Monday if today is not Monday, 
    // or today if it is Monday... wait, 'weekday 1' is Sunday in some systems?
    // SQLite: 0=Sunday, 1=Monday, ..., 6=Saturday.
    // Date('now', '-6 days', 'weekday 1') allows us to go back to previous monday.
    
    // Let's rely on rust-chrono for date calc to be safe and consistent.
    let now = Utc::now().date_naive();
    let days_since_monday = now.weekday().num_days_from_monday();
    let start_date = now - chrono::Duration::days(days_since_monday as i64); // This Monday
    
    let rows = sqlx::query(
        "SELECT date(date_of_sale) as day, SUM(total_resolved) as total, COUNT(*) as count 
         FROM sale_items 
         WHERE date(date_of_sale) >= date(?) 
         GROUP BY day 
         ORDER BY day ASC",
    )
    .bind(start_date.to_string())
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut daily_sales = Vec::new();
    
    // We want to fill in gaps if no sales on a specific day?
    // User requirement: "line chart of sales over the last n days of the week, today inclusive. Assume the week starts on Mondays."
    // So if today is Wed, we want Mon, Tue, Wed.
    

    let today = now;
    
    // Create map for easy lookup
    use std::collections::HashMap;
    let mut sales_map: HashMap<String, (i64, i64)> = HashMap::new(); // (total, count)
    for row in rows {
        let day: String = row.try_get("day").unwrap_or_default();
        let total: i64 = row.try_get("total").unwrap_or(0);
        let count: i64 = row.try_get("count").unwrap_or(0);
        sales_map.insert(day, (total, count));
    }
    
    // Fill the last 7 days (or N days)
    // Actually the requirement is just last week dynamics.
    // Let's iterate from start_date to today.
    
    let mut current_iter = start_date;
    while current_iter <= today {
        let day_str = current_iter.to_string();
        let (total, count) = sales_map.get(&day_str).unwrap_or(&(0, 0));
        daily_sales.push(shared::models::DailySales {
            date: day_str,
            total_sales_cents: *total,
            count: *count,
        });
        current_iter = current_iter + chrono::Duration::days(1);
    }

    Ok(Json(daily_sales))
}

#[derive(Deserialize, IntoParams)]
pub struct StatsRangeParams {
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/sales/stats/top_products",
    tag = "Reports",
    params(
        ("start_date" = Option<String>, Query, description = "Start date YYYY-MM-DD"),
        ("end_date" = Option<String>, Query, description = "End date YYYY-MM-DD")
    ),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get a list of top-selling products within a date range", body = [TopProduct]))
)]
pub async fn get_top_products(
    State(state): State<AppState>,
    Query(params): Query<StatsRangeParams>,
) -> Result<Json<Vec<TopProduct>>, StatusCode> {
    let (start_date, end_date) = get_default_dates(params.start_date, params.end_date);

    let mut query = "
        SELECT p.name as product_name, SUM(s.total_resolved) as total 
        FROM sale_items s
        JOIN products p ON s.product_id = p.id
        WHERE date(s.date_of_sale) >= date(?) AND date(s.date_of_sale) <= date(?)
    ".to_string();
    
    let mut args = Vec::new();
    args.push(start_date);
    args.push(end_date);
    
    query.push_str(" GROUP BY p.name ORDER BY total DESC LIMIT 20");

    let mut sql_query = sqlx::query(&query);
    for arg in &args {
        sql_query = sql_query.bind(arg);
    }

    let rows = sql_query
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;

    let mut results = Vec::new();
    for row in rows {
        let product_name: String = row.try_get("product_name").unwrap_or_default();
        let total: i64 = row.try_get("total").unwrap_or(0);
        results.push(TopProduct {
            product_name,
            total_sales_cents: total,
        });
    }

    Ok(Json(results))
}

#[utoipa::path(
    get,
    path = "/api/sales/stats/by_product",
    tag = "Reports",
    params(StatsRangeParams),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get sales summary grouped by product within a date range", body = [ProductSalesSummary]))
)]
pub async fn get_sales_by_product(
    State(state): State<AppState>,
    Query(params): Query<StatsRangeParams>,
) -> Result<Json<Vec<ProductSalesSummary>>, StatusCode> {
    let (start_date, end_date) = get_default_dates(params.start_date, params.end_date);

    let query = "
        SELECT p.name as product_name, SUM(s.quantity) as total_quantity, SUM(s.total_resolved) as total_amount
        FROM sale_items s
        JOIN products p ON s.product_id = p.id
        WHERE date(s.date_of_sale) >= date(?) AND date(s.date_of_sale) <= date(?)
        GROUP BY p.name
        ORDER BY total_amount DESC
    ";

    let rows = sqlx::query(query)
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;

    let mut results = Vec::new();
    for row in rows {
        results.push(ProductSalesSummary {
            product_name: row.try_get("product_name").unwrap_or_default(),
            total_quantity: row.try_get("total_quantity").unwrap_or(0),
            total_amount_cents: row.try_get("total_amount").unwrap_or(0),
        });
    }

    Ok(Json(results))
}

fn get_default_dates(start: Option<String>, end: Option<String>) -> (String, String) {
    let now = Utc::now();
    let today = now.format("%Y-%m-%d").to_string();
    let first_day = format!("{}-{:02}-01", now.year(), now.month());

    let start_date = start.filter(|s| !s.is_empty()).unwrap_or(first_day);
    let end_date = end.filter(|s| !s.is_empty()).unwrap_or(today);
    
    (start_date, end_date)
}



#[utoipa::path(
    put,
    path = "/api/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    request_body = SaleItemInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Update a specific sale item", body = SaleItem), (status = 404))
)]
pub async fn update_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<SaleItemInput>,
) -> Result<Json<SaleItem>, StatusCode> {
    let sale = SaleItem {
        id,
        sale_id: input.sale_id,
        product_id: input.product_id,
        customer_id: input.customer_id,
        date_of_sale: input.date_of_sale,
        quantity: input.quantity,
        discount: input.discount,
        total_cents: input.total_cents,
        total_resolved: input.total_resolved,
        note: input.note,
        product_name: None,
        price_per_item: None,
    };

    let result = sqlx::query(
        "UPDATE sale_items SET product_id = ?, customer_id = ?, date_of_sale = ?, quantity = ?, discount = ?, total_cents = ?, total_resolved = ?, note = ?, product_name = (SELECT name FROM products WHERE id = ?), price_per_item = (SELECT price_cents FROM products WHERE id = ?) WHERE id = ?",
    )
    .bind(sale.product_id.to_string())
    .bind(sale.customer_id.map(|id| id.to_string()))
    .bind(&sale.date_of_sale)
    .bind(sale.quantity)
    .bind(sale.discount)
    .bind(sale.total_cents)
    .bind(sale.total_resolved)
    .bind(&sale.note)
    .bind(sale.product_id.to_string())
    .bind(sale.product_id.to_string())
    .bind(sale.id.to_string())
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    // Fetch the updated item
    let row = sqlx::query("SELECT sale_items.id, sale_items.sale_id, sale_items.product_id, sale_items.customer_id, sale_items.date_of_sale, sale_items.quantity, sale_items.discount, sale_items.total_cents, sale_items.total_resolved, sale_items.note, COALESCE(sale_items.product_name, products.name) as product_name, COALESCE(sale_items.price_per_item, products.price_cents) as price_per_item FROM sale_items LEFT JOIN products ON sale_items.product_id = products.id WHERE sale_items.id = ?")
        .bind(sale.id.to_string())
        .fetch_one(&state.db)
        .await
        .map_err(map_db_err)?;
    
    let sale = sale_item_from_row(&row)?;

    Ok(Json(sale))
}

#[utoipa::path(
    delete,
    path = "/api/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Delete a specific sale item"), (status = 404))
)]
pub async fn delete_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM sale_items WHERE id = ?")
        .bind(id.to_string())
        .execute(&state.db)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}



#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(utoipa::ToSchema)]
#[allow(dead_code)]
pub struct FileUpload {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Auth",
    request_body = AuthRequest,
    security(()),
    responses((status = 200, description = "Authenticate a staff member and receive a JWT token", body = AuthResponse), (status = 401))
)]
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, password_hash FROM staff WHERE username = ?",
    )
    .bind(&input.username)
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let staff_uuid: String = row.get("id");
    let password_hash: String = row.get("password_hash");

    if !verify_password(&input.password, &state.password_pepper, &password_hash)? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expires_in = 3600;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let claims = Claims {
        sub: staff_uuid,

        exp: (now.as_secs() + expires_in) as usize,
    };
    let key = EncodingKey::from_secret(state.jwt_secret.as_bytes());
    let token = jsonwebtoken::encode(&Header::default(), &claims, &key)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(AuthResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in,
    }))
}

#[utoipa::path(
    get,
    path = "/api/auth/profile",
    tag = "Auth",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve the profile of the currently authenticated staff member", body = Staff))
)]
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Staff>, StatusCode> {
    let staff_uuid = claims.sub;
    let staff = sqlx::query("SELECT * FROM staff WHERE id = ?")
        .bind(staff_uuid)
        .map(|row: SqliteRow| staff_from_row(&row))
        .fetch_one(&state.db)
        .await
        .map_err(map_db_err)?;
    Ok(Json(staff))
}

#[utoipa::path(
    get,
    path = "/api/staff",
    tag = "Staff",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List all staff members", body = [Staff]))
)]
pub async fn list_staff(State(state): State<AppState>) -> Result<Json<Vec<Staff>>, StatusCode> {
    let staff = sqlx::query("SELECT * FROM staff")
        .map(|row: SqliteRow| staff_from_row(&row))
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;
    Ok(Json(staff))
}

#[utoipa::path(
    post,
    path = "/api/staff",
    tag = "Staff",
    request_body = StaffInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Register a new staff member", body = Staff))
)]
pub async fn create_staff(
    State(state): State<AppState>,
    Json(input): Json<StaffInput>,
) -> Result<(StatusCode, Json<Staff>), StatusCode> {
    let password = input.password.ok_or(StatusCode::BAD_REQUEST)?;
    let password_hash = hash_password(&password, &state.password_pepper)?;
    
    let staff = Staff {
        id: Uuid::new_v4(),
        staff_id: input.staff_id,
        first_name: input.first_name,
        last_name: input.last_name,
        mobile_number: input.mobile_number,
        photo_link: input.photo_link,
        username: input.username,
        password_hash,
    };

    sqlx::query(
        "INSERT INTO staff (id, staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(staff.id.to_string())
    .bind(&staff.staff_id)
    .bind(&staff.first_name)
    .bind(&staff.last_name)
    .bind(&staff.mobile_number)
    .bind(&staff.photo_link)
    .bind(&staff.username)
    .bind(&staff.password_hash)
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(staff)))
}

#[utoipa::path(
    get,
    path = "/api/staff/{id}",
    tag = "Staff",
    params(("id" = String, Path, description = "Staff UUID")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve a specific staff member's details", body = Staff), (status = 404))
)]
pub async fn get_staff(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Staff>, StatusCode> {
    let staff_uuid = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let staff = sqlx::query("SELECT * FROM staff WHERE id = ?")
        .bind(staff_uuid.to_string())
        .map(|row: SqliteRow| staff_from_row(&row))
        .fetch_one(&state.db)
        .await
        .map_err(map_db_err)?;
    Ok(Json(staff))
}

#[utoipa::path(
    get,
    path = "/api/staff/{id}/transactions",
    tag = "Staff Transactions",
    params(
        ("id" = String, Path, description = "Staff UUID"),
        ("start_date" = Option<String>, Query, description = "Start date YYYY-MM-DD"),
        ("end_date" = Option<String>, Query, description = "End date YYYY-MM-DD")
    ),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Retrieve sales transactions handled by a specific staff member", body = [Sale]))
)]
pub async fn get_staff_transactions(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(params): Query<StatsRangeParams>,
) -> Result<Json<Vec<Sale>>, StatusCode> {
    let (start_date, end_date) = get_default_dates(params.start_date, params.end_date);

    let mut query = "SELECT * FROM sales WHERE staff_responsible = ? AND date(date_and_time) >= date(?) AND date(date_and_time) <= date(?)".to_string();
    
    query.push_str(" ORDER BY date_and_time DESC");

    let rows = sqlx::query(&query)
        .bind(id.to_string())
        .bind(start_date)
        .bind(end_date)
        .fetch_all(&state.db)
        .await
        .map_err(map_db_err)?;

    let mut sales = Vec::with_capacity(rows.len());
    for row in rows {
        sales.push(sale_from_row(&row)?);
    }

    Ok(Json(sales))
}

#[utoipa::path(
    put,
    path = "/api/staff/{id}",
    tag = "Staff",
    params(("id" = String, Path, description = "Staff UUID")),
    security(("bearer_auth" = [])),
    request_body = StaffInput,
    responses((status = 200, description = "Update a staff member's information", body = Staff), (status = 404))
)]
pub async fn update_staff(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<StaffInput>,
) -> Result<Json<Staff>, StatusCode> {
    let mut tx = state.db.begin().await.map_err(map_db_err)?;
    let staff_uuid = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;

    // Retrieve existing staff to keep password if not updated
    let existing_staff = sqlx::query("SELECT * FROM staff WHERE id = ?")
        .bind(staff_uuid.to_string())
        .map(|row: SqliteRow| staff_from_row(&row))
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_err)?;

    // Handle password update
    let password_hash = if let Some(pwd) = input.password {
        if !pwd.is_empty() {
             hash_password(&pwd, &state.password_pepper)?
        } else {
             existing_staff.password_hash
        }
    } else {
        existing_staff.password_hash
    };

    let updated_staff = Staff {
        id: staff_uuid,
        staff_id: input.staff_id,
        first_name: input.first_name,
        last_name: input.last_name,
        mobile_number: input.mobile_number,
        photo_link: input.photo_link,
        username: input.username,
        password_hash,
    };

    sqlx::query(
        "UPDATE staff SET staff_id = ?, first_name = ?, last_name = ?, mobile_number = ?, photo_link = ?, username = ?, password_hash = ? WHERE id = ?",
    )
    .bind(&updated_staff.staff_id)
    .bind(&updated_staff.first_name)
    .bind(&updated_staff.last_name)
    .bind(&updated_staff.mobile_number)
    .bind(&updated_staff.photo_link)
    .bind(&updated_staff.username)
    .bind(&updated_staff.password_hash)
    .bind(staff_uuid.to_string())
    .execute(&mut *tx)
    .await
    .map_err(map_db_err)?;

    tx.commit().await.map_err(map_db_err)?;

    Ok(Json(updated_staff))
}

#[utoipa::path(
    delete,
    path = "/api/staff/{id}",
    tag = "Staff",
    params(("id" = String, Path, description = "Staff UUID")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Permanently remove a staff member"), (status = 404))
)]
pub async fn delete_staff(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let staff_uuid = Uuid::parse_str(&id).map_err(|_| StatusCode::BAD_REQUEST)?;
    let result = sqlx::query("DELETE FROM staff WHERE id = ?")
        .bind(staff_uuid.to_string())
        .execute(&state.db)
        .await
        .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        Ok(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

fn product_from_row(row: &SqliteRow) -> Result<Product, StatusCode> {
    let product_type: String = row.get("product_type");
    let product_type = ProductType::from_str(&product_type)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Product {
        id: parse_uuid(row.get("id"))?,
        name: row.get("name"),
        description: row.get("description"),
        price_cents: row.get("price_cents"),
        stock: row.get("stock"),
        product_type,
        details: vec![],
    })
}

fn customer_from_row(row: &SqliteRow) -> Result<Customer, StatusCode> {
    Ok(Customer {
        id: parse_uuid(row.get("id"))?,
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        middle_name: row.get("middle_name"),
        mobile_number: row.get("mobile_number"),
        date_of_birth: row.get("date_of_birth"),
        email: row.get("email"),
        details: vec![],
    })
}

fn sale_item_from_row(row: &SqliteRow) -> Result<SaleItem, StatusCode> {
    let sale_id_str: Option<String> = row.get("sale_id");
    let sale_id = match sale_id_str {
        Some(s) => Some(parse_uuid(s)?),
        None => None,
    };
    
    let customer_id_str: Option<String> = row.get("customer_id");
    let customer_id = match customer_id_str {
        Some(s) => Some(parse_uuid(s)?),
        None => None,
    };

    Ok(SaleItem {
        id: parse_uuid(row.get("id"))?,
        sale_id,
        product_id: parse_uuid(row.get("product_id"))?,
        customer_id,
        date_of_sale: row.get("date_of_sale"),
        quantity: row.get("quantity"),
        discount: row.get("discount"),
        total_cents: row.get("total_cents"),
        total_resolved: row.get("total_resolved"),
        note: row.get("note"),
        product_name: row.try_get("product_name").ok(),
        price_per_item: row.try_get("price_per_item").ok(),
    })
}

fn sale_from_row(row: &SqliteRow) -> Result<Sale, StatusCode> {
    let sales_channel_str: String = row.get("sales_channel");
    let sales_channel = shared::models::SalesChannel::from_str(&sales_channel_str).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let customer_id_str: Option<String> = row.get("customer_id");
    let customer_id = match customer_id_str {
        Some(s) => Some(parse_uuid(s)?),
        None => None,
    };

    Ok(Sale {
        id: parse_uuid(row.get("id"))?,
        customer_id,
        date_and_time: row.get("date_and_time"),
        sale_items: vec![],
        total_cents: row.get("total_cents"),
        discount: row.get("discount"),
        total_resolved: row.get("total_resolved"),
        sales_channel,
        staff_responsible: parse_uuid(row.get("staff_responsible"))?,
        company_branch: row.get("company_branch"),
        car_number: row.get("car_number"),
        receipt_number: row.get("receipt_number"),
    })
}

fn staff_from_row(row: &SqliteRow) -> Staff {
    Staff {
        id: Uuid::parse_str(row.get("id")).unwrap_or_default(),
        staff_id: row.get("staff_id"),
        first_name: row.get("first_name"),
        last_name: row.get("last_name"),
        mobile_number: row.get("mobile_number"),
        photo_link: row.get("photo_link"),
        username: row.get("username"),
        password_hash: row.get("password_hash"),
    }
}

fn parse_uuid(value: String) -> Result<Uuid, StatusCode> {
    Uuid::parse_str(&value).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn map_db_err(err: sqlx::Error) -> StatusCode {
    match err {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn hash_password(password: &str, pepper: &str) -> Result<String, StatusCode> {
    let salted = format!("{password}{pepper}");
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(salted.as_bytes(), &salt)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .to_string();
    Ok(hash)
}

fn verify_password(password: &str, pepper: &str, stored: &str) -> Result<bool, StatusCode> {
    let salted = format!("{password}{pepper}");
    let parsed =
        PasswordHash::new(stored).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let argon2 = Argon2::default();
    Ok(argon2
        .verify_password(salted.as_bytes(), &parsed)
        .is_ok())
}



#[utoipa::path(
    post,
    path = "/api/upload",
    tag = "Utility",
    request_body(content = FileUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "File uploaded", body = UploadResponse),
        (status = 400, description = "Bad Request")
    )
)]
pub async fn upload_file(mut multipart: axum::extract::Multipart) -> Result<Json<UploadResponse>, StatusCode> {
    while let Some(field) = multipart.next_field().await.map_err(|_| StatusCode::BAD_REQUEST)? {
        let file_name = field.file_name().unwrap_or("file").to_string();
        
        if let Some(ext) = std::path::Path::new(&file_name).extension().and_then(|s| s.to_str()) {
             let data = field.bytes().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
             
             let new_filename = format!("{}.{}", Uuid::new_v4(), ext);
             let upload_dir = "uploads";
             tokio::fs::create_dir_all(upload_dir).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
             
             let filepath = std::path::Path::new(upload_dir).join(&new_filename);
             tokio::fs::write(&filepath, data).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
             
             return Ok(Json(UploadResponse {
                 url: format!("/uploads/{}", new_filename),
             }));
        }
    }
    Err(StatusCode::BAD_REQUEST)
}
