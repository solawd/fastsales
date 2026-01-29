use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::NaiveDate;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use jsonwebtoken::{EncodingKey, Header};
use sqlx::sqlite::SqliteRow;
use sqlx::Row;
use rand_core::OsRng;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::AppState;
use crate::auth::Claims;
use crate::models::{
    Customer, CustomerInput, Product, ProductInput, ProductType, Sale, SaleInput, Staff, StaffInput,
};

#[utoipa::path(
    get,
    path = "/products",
    tag = "Products",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List products", body = [Product]))
)]
pub async fn list_products(State(state): State<AppState>) -> Result<Json<Vec<Product>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, name, description, price_cents, stock, product_type FROM products",
    )
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut products = Vec::with_capacity(rows.len());
    for row in rows {
        products.push(product_from_row(&row)?);
    }
    Ok(Json(products))
}

#[utoipa::path(
    post,
    path = "/products",
    tag = "Products",
    request_body = ProductInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Product created", body = Product))
)]
pub async fn create_product(
    State(state): State<AppState>,
    Json(input): Json<ProductInput>,
) -> Result<(StatusCode, Json<Product>), StatusCode> {
    let product = Product {
        id: Uuid::new_v4(),
        name: input.name,
        description: input.description,
        price_cents: input.price_cents,
        stock: input.stock,
        product_type: input.product_type,
    };

    sqlx::query(
        "INSERT INTO products (id, name, description, price_cents, stock, product_type) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(product.id.to_string())
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price_cents)
    .bind(product.stock)
    .bind(product.product_type.as_str())
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(product)))
}

#[utoipa::path(
    get,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get product", body = Product), (status = 404))
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

    Ok(Json(product_from_row(&row)?))
}

#[utoipa::path(
    put,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    request_body = ProductInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Product updated", body = Product), (status = 404))
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
    };

    let result = sqlx::query(
        "UPDATE products SET name = ?, description = ?, price_cents = ?, stock = ?, product_type = ? WHERE id = ?",
    )
    .bind(&product.name)
    .bind(&product.description)
    .bind(product.price_cents)
    .bind(product.stock)
    .bind(product.product_type.as_str())
    .bind(product.id.to_string())
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(product))
}

#[utoipa::path(
    delete,
    path = "/products/{id}",
    tag = "Products",
    params(("id" = String, Path, description = "Product id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Product deleted"), (status = 404))
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
    path = "/customers",
    tag = "Customers",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List customers", body = [Customer]))
)]
pub async fn list_customers(State(state): State<AppState>) -> Result<Json<Vec<Customer>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, first_name, last_name, middle_name, mobile_number, date_of_birth, email FROM customers",
    )
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut customers = Vec::with_capacity(rows.len());
    for row in rows {
        customers.push(customer_from_row(&row)?);
    }
    Ok(Json(customers))
}

#[utoipa::path(
    post,
    path = "/customers",
    tag = "Customers",
    request_body = CustomerInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Customer created", body = Customer), (status = 400))
)]
pub async fn create_customer(
    State(state): State<AppState>,
    Json(input): Json<CustomerInput>,
) -> Result<(StatusCode, Json<Customer>), StatusCode> {
    if !is_valid_date_of_birth(&input.date_of_birth) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let customer = Customer {
        id: Uuid::new_v4(),
        first_name: input.first_name,
        last_name: input.last_name,
        middle_name: input.middle_name,
        mobile_number: input.mobile_number,
        date_of_birth: input.date_of_birth,
        email: input.email,
    };

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
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(customer)))
}

#[utoipa::path(
    get,
    path = "/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get customer", body = Customer), (status = 404))
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

    Ok(Json(customer_from_row(&row)?))
}

#[utoipa::path(
    put,
    path = "/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    request_body = CustomerInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Customer updated", body = Customer), (status = 400), (status = 404))
)]
pub async fn update_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<CustomerInput>,
) -> Result<Json<Customer>, StatusCode> {
    if !is_valid_date_of_birth(&input.date_of_birth) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let customer = Customer {
        id,
        first_name: input.first_name,
        last_name: input.last_name,
        middle_name: input.middle_name,
        mobile_number: input.mobile_number,
        date_of_birth: input.date_of_birth,
        email: input.email,
    };

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
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(customer))
}

#[utoipa::path(
    delete,
    path = "/customers/{id}",
    tag = "Customers",
    params(("id" = String, Path, description = "Customer id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Customer deleted"), (status = 404))
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
    path = "/sales",
    tag = "Sales",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List sales", body = [Sale]))
)]
pub async fn list_sales(State(state): State<AppState>) -> Result<Json<Vec<Sale>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note FROM sales",
    )
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
    post,
    path = "/sales",
    tag = "Sales",
    request_body = SaleInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Sale created", body = Sale))
)]
pub async fn create_sale(
    State(state): State<AppState>,
    Json(input): Json<SaleInput>,
) -> Result<(StatusCode, Json<Sale>), StatusCode> {
    let sale = Sale {
        id: Uuid::new_v4(),
        product_id: input.product_id,
        customer_id: input.customer_id,
        date_of_sale: input.date_of_sale,
        quantity: input.quantity,
        discount: input.discount,
        total_cents: input.total_cents,
        total_resolved: input.total_resolved,
        note: input.note,
    };

    sqlx::query(
        "INSERT INTO sales (id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(sale.id.to_string())
    .bind(sale.product_id.to_string())
    .bind(sale.customer_id.to_string())
    .bind(&sale.date_of_sale)
    .bind(sale.quantity)
    .bind(sale.discount)
    .bind(sale.total_cents)
    .bind(if sale.total_resolved { 1 } else { 0 })
    .bind(&sale.note)
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    Ok((StatusCode::CREATED, Json(sale)))
}

#[utoipa::path(
    get,
    path = "/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get sale", body = Sale), (status = 404))
)]
pub async fn get_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Sale>, StatusCode> {
    let row = sqlx::query(
        "SELECT id, product_id, customer_id, date_of_sale, quantity, discount, total_cents, total_resolved, note FROM sales WHERE id = ?",
    )
    .bind(id.to_string())
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(sale_from_row(&row)?))
}

#[utoipa::path(
    put,
    path = "/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    request_body = SaleInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Sale updated", body = Sale), (status = 404))
)]
pub async fn update_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(input): Json<SaleInput>,
) -> Result<Json<Sale>, StatusCode> {
    let sale = Sale {
        id,
        product_id: input.product_id,
        customer_id: input.customer_id,
        date_of_sale: input.date_of_sale,
        quantity: input.quantity,
        discount: input.discount,
        total_cents: input.total_cents,
        total_resolved: input.total_resolved,
        note: input.note,
    };

    let result = sqlx::query(
        "UPDATE sales SET product_id = ?, customer_id = ?, date_of_sale = ?, quantity = ?, discount = ?, total_cents = ?, total_resolved = ?, note = ? WHERE id = ?",
    )
    .bind(sale.product_id.to_string())
    .bind(sale.customer_id.to_string())
    .bind(&sale.date_of_sale)
    .bind(sale.quantity)
    .bind(sale.discount)
    .bind(sale.total_cents)
    .bind(if sale.total_resolved { 1 } else { 0 })
    .bind(&sale.note)
    .bind(sale.id.to_string())
    .execute(&state.db)
    .await
    .map_err(map_db_err)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }

    Ok(Json(sale))
}

#[utoipa::path(
    delete,
    path = "/sales/{id}",
    tag = "Sales",
    params(("id" = String, Path, description = "Sale id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Sale deleted"), (status = 404))
)]
pub async fn delete_sale(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM sales WHERE id = ?")
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

fn is_valid_date_of_birth(value: &str) -> bool {
    NaiveDate::parse_from_str(value, "%d-%m-%Y").is_ok()
}

#[derive(serde::Deserialize, utoipa::ToSchema)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct AuthResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: u64,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = AuthRequest,
    security(()),
    responses((status = 200, description = "Login success", body = AuthResponse), (status = 401))
)]
pub async fn login(
    State(state): State<AppState>,
    Json(input): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let row = sqlx::query(
        "SELECT staff_id, password_hash FROM staff WHERE username = ?",
    )
    .bind(&input.username)
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::UNAUTHORIZED)?;

    let staff_id: String = row.get("staff_id");
    let password_hash: String = row.get("password_hash");

    if !verify_password(&input.password, &state.password_pepper, &password_hash)? {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let expires_in = 3600;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let claims = Claims {
        sub: staff_id,
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
    path = "/staff",
    tag = "Staff",
    security(("bearer_auth" = [])),
    responses((status = 200, description = "List staff", body = [Staff]))
)]
pub async fn list_staff(State(state): State<AppState>) -> Result<Json<Vec<Staff>>, StatusCode> {
    let rows = sqlx::query(
        "SELECT staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash FROM staff",
    )
    .fetch_all(&state.db)
    .await
    .map_err(map_db_err)?;

    let mut staff = Vec::with_capacity(rows.len());
    for row in rows {
        staff.push(staff_from_row(&row));
    }
    Ok(Json(staff))
}

#[utoipa::path(
    post,
    path = "/staff",
    tag = "Staff",
    request_body = StaffInput,
    security(("bearer_auth" = [])),
    responses((status = 201, description = "Staff created", body = Staff))
)]
pub async fn create_staff(
    State(state): State<AppState>,
    Json(input): Json<StaffInput>,
) -> Result<(StatusCode, Json<Staff>), StatusCode> {
    let password_hash = hash_password(&input.password, &state.password_pepper)?;
    let staff = Staff {
        first_name: input.first_name,
        last_name: input.last_name,
        mobile_number: input.mobile_number,
        photo_link: input.photo_link,
        staff_id: input.staff_id,
        username: input.username,
        password_hash,
    };

    sqlx::query(
        "INSERT INTO staff (staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
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
    path = "/staff/{staff_id}",
    tag = "Staff",
    params(("staff_id" = String, Path, description = "Staff id")),
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Get staff", body = Staff), (status = 404))
)]
pub async fn get_staff(
    State(state): State<AppState>,
    Path(staff_id): Path<String>,
) -> Result<Json<Staff>, StatusCode> {
    let row = sqlx::query(
        "SELECT staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash FROM staff WHERE staff_id = ?",
    )
    .bind(&staff_id)
    .fetch_optional(&state.db)
    .await
    .map_err(map_db_err)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(staff_from_row(&row)))
}

#[utoipa::path(
    put,
    path = "/staff/{staff_id}",
    tag = "Staff",
    params(("staff_id" = String, Path, description = "Staff id")),
    request_body = StaffInput,
    security(("bearer_auth" = [])),
    responses((status = 200, description = "Staff updated", body = Staff), (status = 404))
)]
pub async fn update_staff(
    State(state): State<AppState>,
    Path(staff_id): Path<String>,
    Json(input): Json<StaffInput>,
) -> Result<Json<Staff>, StatusCode> {
    let password_hash = hash_password(&input.password, &state.password_pepper)?;
    let staff = Staff {
        first_name: input.first_name,
        last_name: input.last_name,
        mobile_number: input.mobile_number,
        photo_link: input.photo_link,
        staff_id: input.staff_id,
        username: input.username,
        password_hash,
    };

    let mut tx = state.db.begin().await.map_err(map_db_err)?;

    let exists = sqlx::query("SELECT staff_id FROM staff WHERE staff_id = ?")
        .bind(&staff_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_err)?
        .is_some();
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }

    if staff.staff_id == staff_id {
        sqlx::query(
            "UPDATE staff SET first_name = ?, last_name = ?, mobile_number = ?, photo_link = ?, username = ?, password_hash = ? WHERE staff_id = ?",
        )
        .bind(&staff.first_name)
        .bind(&staff.last_name)
        .bind(&staff.mobile_number)
        .bind(&staff.photo_link)
        .bind(&staff.username)
        .bind(&staff.password_hash)
        .bind(&staff_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    } else {
        sqlx::query("DELETE FROM staff WHERE staff_id = ?")
            .bind(&staff_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_err)?;
        sqlx::query(
            "INSERT INTO staff (staff_id, first_name, last_name, mobile_number, photo_link, username, password_hash) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&staff.staff_id)
        .bind(&staff.first_name)
        .bind(&staff.last_name)
        .bind(&staff.mobile_number)
        .bind(&staff.photo_link)
        .bind(&staff.username)
        .bind(&staff.password_hash)
        .execute(&mut *tx)
        .await
        .map_err(map_db_err)?;
    }

    tx.commit().await.map_err(map_db_err)?;

    Ok(Json(staff))
}

#[utoipa::path(
    delete,
    path = "/staff/{staff_id}",
    tag = "Staff",
    params(("staff_id" = String, Path, description = "Staff id")),
    security(("bearer_auth" = [])),
    responses((status = 204, description = "Staff deleted"), (status = 404))
)]
pub async fn delete_staff(
    State(state): State<AppState>,
    Path(staff_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM staff WHERE staff_id = ?")
        .bind(&staff_id)
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
    })
}

fn sale_from_row(row: &SqliteRow) -> Result<Sale, StatusCode> {
    let total_resolved: i64 = row.get("total_resolved");

    Ok(Sale {
        id: parse_uuid(row.get("id"))?,
        product_id: parse_uuid(row.get("product_id"))?,
        customer_id: parse_uuid(row.get("customer_id"))?,
        date_of_sale: row.get("date_of_sale"),
        quantity: row.get("quantity"),
        discount: row.get("discount"),
        total_cents: row.get("total_cents"),
        total_resolved: total_resolved != 0,
        note: row.get("note"),
    })
}

fn staff_from_row(row: &SqliteRow) -> Staff {
    Staff {
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
