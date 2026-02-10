use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub stock: i64,
    pub product_type: ProductType,
    #[schema(no_recursion)]
    pub details: Vec<ProductDetails>,
}

impl std::fmt::Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ProductInput {
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub stock: i64,
    pub product_type: ProductType,
    #[schema(no_recursion)]
    pub details: Vec<ProductDetailsInput>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductDetails {
    pub product_id: Uuid,
    pub detail_name: String,
    pub detail_value: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct ProductDetailsInput {
    pub detail_name: String,
    pub detail_value: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProductType {
    PhysicalGood,
    Service,
}

impl ProductType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProductType::PhysicalGood => "physical_good",
            ProductType::Service => "service",
        }
    }
}

impl FromStr for ProductType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "physical_good" => Ok(ProductType::PhysicalGood),
            "service" => Ok(ProductType::Service),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Customer {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub mobile_number: String,
    pub date_of_birth: NaiveDate,
    pub email: String,
    #[schema(no_recursion)]
    pub details: Vec<CustomerDetails>,
}

impl std::fmt::Display for Customer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.first_name, self.last_name)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct CustomerInput {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub mobile_number: String,
    pub date_of_birth: NaiveDate,
    pub email: String,
    #[schema(no_recursion)]
    pub details: Vec<CustomerDetailsInput>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomerDetails {
    pub customer_id: Uuid,
    pub detail_name: String,
    pub detail_value: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct CustomerDetailsInput {
    pub detail_name: String,
    pub detail_value: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct UploadResponse {
    pub url: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema, Debug)]
pub struct SaleItem {
    pub id: Uuid,
    pub sale_id: Option<Uuid>, // New field, optional for now to support legacy items? Or should be linking to parent Sale.
    pub product_id: Uuid,
    pub customer_id: Uuid,
    pub date_of_sale: DateTime<Utc>,
    pub quantity: i64,
    pub discount: i64,
    pub total_cents: i64,
    pub total_resolved: i64, // Amount resolved in cents
    pub note: Option<String>,
    pub product_name: Option<String>,
    pub price_per_item: Option<i64>,
}

impl std::fmt::Display for SaleItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.product_name {
            write!(f, "{}x {}", self.quantity, name)
        } else {
            write!(f, "{}x Product {}", self.quantity, self.product_id)
        }
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct SaleItemInput {
    pub sale_id: Option<Uuid>,
    pub product_id: Uuid,
    pub customer_id: Uuid,
    pub date_of_sale: DateTime<Utc>,
    pub quantity: i64,
    pub discount: i64,
    pub total_cents: i64,
    pub total_resolved: i64, // Amount resolved in cents
    pub note: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SalesChannel {
    Mobile,
    Web,
}

impl std::fmt::Display for SalesChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            SalesChannel::Mobile => "mobile",
            SalesChannel::Web => "web",
        };
        write!(f, "{}", s)
    }
}

impl FromStr for SalesChannel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mobile" => Ok(SalesChannel::Mobile),
            "web" => Ok(SalesChannel::Web),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Serialize, Deserialize, ToSchema, Debug)]
pub struct Sale {
    pub id: Uuid,
    pub customer_id: Option<Uuid>,
    pub date_and_time: DateTime<Utc>,
    pub sale_items: Vec<SaleItem>,
    pub total_cents: i64,
    pub discount: i64,
    pub total_resolved: i64,
    pub sales_channel: SalesChannel,
    pub staff_responsible: Uuid,
    pub company_branch: String,
    pub car_number: String,
    pub receipt_number: String,
}

impl std::fmt::Display for Sale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sale #{}", self.receipt_number)
    }
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct SaleInput {
    pub customer_id: Option<Uuid>,
    pub date_and_time: DateTime<Utc>,
    pub sale_items: Vec<SaleItemInput>,
    pub total_cents: i64,
    pub discount: i64,
    pub total_resolved: i64,
    pub sales_channel: SalesChannel,
    pub staff_responsible: Uuid,
    pub company_branch: String,
    pub car_number: String,
    pub receipt_number: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Staff {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub mobile_number: String,
    pub photo_link: String,
    pub staff_id: String,
    pub username: String,
    pub password_hash: String,
}

impl std::fmt::Display for Staff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.first_name, self.last_name)
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct StaffInput {
    pub id: Option<Uuid>,
    pub first_name: String,
    pub last_name: String,
    pub mobile_number: String,
    pub photo_link: String,
    pub staff_id: String,
    pub username: String,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Default)]
pub struct SalesStats {
    pub total_sales_cents: i64,
    pub count: i64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct DailySales {
    pub date: String, // YYYY-MM-DD
    pub total_sales_cents: i64,
    pub count: i64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct SalesItemsListResponse {
    pub sales: Vec<SaleItem>,
    pub total_sales_period_cents: i64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct TopProduct {
    pub product_name: String,
    pub total_sales_cents: i64,
}

#[derive(Serialize, Deserialize, ToSchema, Clone, Debug)]
pub struct ProductSalesSummary {
    pub product_name: String,
    pub total_quantity: i64,
    pub total_amount_cents: i64,
}
