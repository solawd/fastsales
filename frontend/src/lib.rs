use leptos::*;
use leptos_meta::{Meta, Stylesheet, Title, provide_meta_context};
use leptos_router::{Route, Router, Routes};

pub mod components;
use components::layout::DashboardLayout;
pub mod pages;

use pages::login::LoginPage;
use pages::home::DashboardPage;
use pages::products::{ProductListPage, ProductEditPage};
use pages::sales::{SalesListPage, SalesEditPage};
use pages::customers::{CustomersListPage, CustomerEditPage};
use pages::staff::{StaffListPage, StaffEditPage};
use pages::profile::ProfilePage;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Title text="FastSales"/>
        <Meta name="description" content="FastSales API + Leptos UI"/>
        <Stylesheet id="leptos" href="/pkg/fastsales.css"/>

        <Router>
            <main>
                <Routes>
                    <Route path="/" view=LoginPage/>
                    <Route path="" view=DashboardLayout>
                        <Route path="/dashboard" view=DashboardPage/>
                        <Route path="/profile" view=ProfilePage/>
                        <Route path="/products" view=ProductListPage/>
                        <Route path="/products/create" view=ProductEditPage/>
                        <Route path="/products/:id" view=ProductEditPage/>
                        <Route path="/sales" view=SalesListPage/>
                        <Route path="/sales/create" view=SalesEditPage/>
                        <Route path="/sales/:id" view=SalesEditPage/>
                        <Route path="/customers" view=CustomersListPage/>
                        <Route path="/customers/create" view=CustomerEditPage/>
                        <Route path="/customers/:id" view=CustomerEditPage/>
                        <Route path="/staff" view=StaffListPage/>
                        <Route path="/staff/create" view=StaffEditPage/>
                        <Route path="/staff/:id" view=StaffEditPage/>
                    </Route>
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use leptos::logging::log;
    log!("HYDRATING FRONTEND");
    leptos::mount_to_body(App);
}
