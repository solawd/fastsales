use axum::{
    Router,
    middleware::from_fn_with_state,
    routing::{get, post},
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::env;
use utoipa::{Modify, OpenApi};
use utoipa::openapi::security::{Http, HttpAuthScheme, SecurityScheme};
use utoipa_swagger_ui::SwaggerUi;

mod models;
mod handlers;
mod auth;

use handlers::{
    create_customer, create_product, create_sale, create_staff, delete_customer, delete_product,
    delete_sale, delete_staff, get_customer, get_product, get_sale, get_staff, list_customers,
    list_products, list_sales, list_staff, update_customer, update_product, update_sale,
    update_staff, login,
};
use auth::auth_middleware;
use sqlx::SqlitePool;

#[derive(OpenApi)]
#[openapi(
    paths(
        handlers::list_products,
        handlers::create_product,
        handlers::get_product,
        handlers::update_product,
        handlers::delete_product,
        handlers::list_customers,
        handlers::create_customer,
        handlers::get_customer,
        handlers::update_customer,
        handlers::delete_customer,
        handlers::list_sales,
        handlers::create_sale,
        handlers::get_sale,
        handlers::update_sale,
        handlers::delete_sale,
        handlers::list_staff,
        handlers::create_staff,
        handlers::get_staff,
        handlers::update_staff,
        handlers::delete_staff,
        handlers::login
    ),
    components(schemas(
        models::Product,
        models::ProductInput,
        models::ProductType,
        models::Customer,
        models::CustomerInput,
        models::Sale,
        models::SaleInput,
        models::Staff,
        models::StaffInput,
        handlers::AuthRequest,
        handlers::AuthResponse
    )),
    tags(
        (name = "Products", description = "Product CRUD"),
        (name = "Customers", description = "Customer CRUD"),
        (name = "Sales", description = "Sales CRUD"),
        (name = "Staff", description = "Staff CRUD"),
        (name = "Auth", description = "Authentication")
    ),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let scheme = SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer));
        openapi
            .components
            .get_or_insert_with(Default::default)
            .add_security_scheme("bearer_auth", scheme);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub jwt_secret: String,
    pub password_pepper: String,
}

#[tokio::main]
async fn main() {
    let pool = if let Ok(database_url) = env::var("DATABASE_URL") {
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("database connection failed")
    } else {
        let db_path = env::current_dir()
            .expect("current dir unavailable")
            .join("fastsales.db");
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);
        SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .expect("database connection failed")
    };
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("database migrations failed");

    let state = AppState {
        db: pool,
        jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret".to_string()),
        password_pepper: env::var("PASSWORD_PEPPER").unwrap_or_default(),
    };

    let protected = Router::new()
        .route("/products", get(list_products).post(create_product))
        .route(
            "/products/:id",
            get(get_product).put(update_product).delete(delete_product),
        )
        .route("/customers", get(list_customers).post(create_customer))
        .route(
            "/customers/:id",
            get(get_customer).put(update_customer).delete(delete_customer),
        )
        .route("/sales", get(list_sales).post(create_sale))
        .route(
            "/sales/:id",
            get(get_sale).put(update_sale).delete(delete_sale),
        )
        .route("/staff", get(list_staff).post(create_staff))
        .route(
            "/staff/:id",
            get(get_staff).put(update_staff).delete(delete_staff),
        )
        .route_layer(from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state.clone());

    let app = Router::new()
        .route("/auth/login", post(login))
        .merge(protected)
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-doc/openapi.json", ApiDoc::openapi()),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("listener bind failed");
    axum::serve(listener, app).await.expect("server failed");
}
