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
}

#[derive(Deserialize, ToSchema)]
pub struct ProductInput {
    pub name: String,
    pub description: String,
    pub price_cents: i64,
    pub stock: i64,
    pub product_type: ProductType,
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
    pub date_of_birth: String,
    pub email: String,
}

#[derive(Deserialize, ToSchema)]
pub struct CustomerInput {
    pub first_name: String,
    pub last_name: String,
    pub middle_name: Option<String>,
    pub mobile_number: String,
    pub date_of_birth: String,
    pub email: String,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Sale {
    pub id: Uuid,
    pub product_id: Uuid,
    pub customer_id: Uuid,
    pub date_of_sale: String,
    pub quantity: i64,
    pub discount: i64,
    pub total_cents: i64,
    pub total_resolved: bool,
    pub note: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct SaleInput {
    pub product_id: Uuid,
    pub customer_id: Uuid,
    pub date_of_sale: String,
    pub quantity: i64,
    pub discount: i64,
    pub total_cents: i64,
    pub total_resolved: bool,
    pub note: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct Staff {
    pub first_name: String,
    pub last_name: String,
    pub mobile_number: String,
    pub photo_link: String,
    pub staff_id: String,
    pub username: String,
    pub password_hash: String,
}

#[derive(Deserialize, ToSchema)]
pub struct StaffInput {
    pub first_name: String,
    pub last_name: String,
    pub mobile_number: String,
    pub photo_link: String,
    pub staff_id: String,
    pub username: String,
    pub password: String,
}
