use leptos::*;
use leptos_router::*;
use shared::models::{Sale, Customer, Product, SaleInput};
#[cfg(target_arch = "wasm32")]
use shared::models::SalesListResponse;
use uuid::Uuid;
use chrono::{Utc, NaiveDate};

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::utils::CURRENCY;

#[component]
pub fn SalesListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (sales, set_sales) = create_signal(Vec::<Sale>::new());
    let (total_period_sales, _set_total_period_sales) = create_signal(0i64);
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    
    let navigate = use_navigate();
    let fetch_sales = move || {
        let _navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
            spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                
                let mut url = "/api/sales".to_string();
                let mut params = Vec::new();
                if !start_date.get().is_empty() {
                    params.push(format!("start_date={}", start_date.get()));
                }
                if !end_date.get().is_empty() {
                    params.push(format!("end_date={}", end_date.get()));
                }
                if !params.is_empty() {
                    url.push('?');
                    url.push_str(&params.join("&"));
                }

                if let Ok(res) = Request::get(&url)
                    .header("Authorization", &format!("Bearer {}", token))
                    .send().await {
                    if res.status() == 401 {
                        _navigate("/", Default::default());
                        return;
                    }
                    if let Ok(data) = res.json::<SalesListResponse>().await {
                        set_sales.set(data.sales);
                        _set_total_period_sales.set(data.total_sales_period_cents);
                    }
                }
            });
    };

    create_effect({
        let fetch_sales = fetch_sales.clone();
        move |_| {
            fetch_sales();
        }
    });

    let delete_action = {
        let fetch_sales = fetch_sales.clone();
        move |id: Uuid| {
            let _fetch_sales = fetch_sales.clone();
            #[allow(unused_variables)]
            let id = id;
            #[cfg(target_arch = "wasm32")]
            spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                let _ = Request::delete(&format!("/api/sales/{}", id))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await;
                _fetch_sales();
            });
        }
    };

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Sales"</h1>
                <A href="/sales/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "Add Sale"
                </A>
            </div>

            <div style="background: var(--bg-surface); padding: 1rem; border-radius: var(--radius-md); border: 1px solid var(--border-subtle); margin-bottom: 2rem;">
                <div style="display: flex; gap: 1rem; align-items: flex-end;">
                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500; font-size: 0.9rem;">"Start Date"</label>
                        <input 
                            type="date" 
                            prop:value=start_date
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500; font-size: 0.9rem;">"End Date"</label>
                        <input 
                            type="date" 
                            prop:value=end_date
                            on:input=move |ev| set_end_date.set(event_target_value(&ev))
                        />
                    </div>
                    <button 
                        class="btn-primary"
                        on:click={
                            let fetch_sales = fetch_sales.clone();
                            move |_| fetch_sales()
                        }
                        style="padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); border: none; font-weight: 600; cursor: pointer;"
                    >
                        "Filter"
                    </button>
                    
                    <div style="margin-left: auto; text-align: right;">
                         <span style="display: block; font-size: 0.9rem; color: var(--text-muted); margin-bottom: 0.25rem;">"Total Sales for Period"</span>
                         <span style="font-size: 1.5rem; font-weight: 700; color: var(--brand-primary);">
                            {move || format!("{}{:.2}", CURRENCY, total_period_sales.get() as f64 / 100.0)}
                         </span>
                    </div>
                </div>
            </div>



            <div style="overflow-x: auto; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background-color: var(--bg-subtle); text-align: left;">
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Date"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">{format!("Total ({})", CURRENCY)}</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Resolved Amount"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || sales.get()
                            key=|sale| sale.id
                            children=move |sale| {
                                let _s_id = sale.id;
                                let _delete_handler = delete_action.clone();
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">{sale.date_of_sale.format("%Y-%m-%d").to_string()}</td>
                                        <td style="padding: 1rem;">{format!("{:.2}", sale.total_cents as f64 / 100.0)}</td>
                                        <td style="padding: 1rem;">
                                            
                                            <span style="font-size: 0.8em; color: var(--text-muted); margin-left: 0.5rem;">
                                                {format!("{:.2}", sale.total_resolved as f64 / 100.0)}
                                            </span>
                                        </td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/sales/{}", sale.id) attr:style="color: var(--brand-primary); text-decoration: none; font-weight: 500;">"Edit"</A>
                                            <button 
                                                on:click=move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if web_sys::window().unwrap().confirm_with_message("Are you sure?").unwrap() {
                                                        _delete_handler(_s_id);
                                                    }
                                                }
                                                style="background: none; border: none; color: var(--state-error); cursor: pointer; font-weight: 500;"
                                            >
                                                "Delete"
                                            </button>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
            </div>
        </div>
    }
}

#[component]
pub fn SalesEditPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();
    let is_create = move || id() == "create" || id().is_empty();

    // Form State
    let (product_id, set_product_id) = create_signal(String::new());
    let (customer_id, set_customer_id) = create_signal(String::new());
    let (date_of_sale, set_date_of_sale) = create_signal(String::new());
    let (quantity, set_quantity) = create_signal(1);
    let (discount, set_discount) = create_signal(0.0);
    let (total, set_total) = create_signal(0.0);
    let (resolved_amount, set_resolved_amount) = create_signal(0.0);
    let (note, set_note) = create_signal(String::new());

    // Dropdown Data
    #[allow(unused_variables)]
    let (products, set_products) = create_signal(Vec::<Product>::new());
    #[allow(unused_variables)]
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());

    #[allow(unused_variables)]
    let navigate = use_navigate();

    // Fetch Lists and Sale Data
    create_effect(move |_| {
        let _current_id = id();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            // Fetch Products
            if let Ok(res) = Request::get("/api/products").header("Authorization", &format!("Bearer {}", token)).send().await {
                if let Ok(data) = res.json::<Vec<Product>>().await {
                    set_products.set(data);
                }
            }
            
            // Fetch Customers (Assuming endpoint exists, per plan)
            if let Ok(res) = Request::get("/api/customers").header("Authorization", &format!("Bearer {}", token)).send().await {
                if let Ok(data) = res.json::<Vec<Customer>>().await {
                    set_customers.set(data);
                }
            }

            // Fetch Sale if Editing
            if _current_id != "create" && !_current_id.is_empty() {
                if let Ok(res) = Request::get(&format!("/api/sales/{}", _current_id)).header("Authorization", &format!("Bearer {}", token)).send().await {
                    if let Ok(sale) = res.json::<Sale>().await {
                        set_product_id.set(sale.product_id.to_string());
                        set_customer_id.set(sale.customer_id.to_string());
                        set_date_of_sale.set(sale.date_of_sale.format("%Y-%m-%d").to_string());
                        set_quantity.set(sale.quantity);
                        set_discount.set(sale.discount as f64 / 100.0);
                        set_total.set(sale.total_cents as f64 / 100.0);
                        set_resolved_amount.set(sale.total_resolved as f64 / 100.0);
                        set_note.set(sale.note.unwrap_or_default());
                    }
                }
            }
        });
    });

    let save_sale = move |_| {
        #[allow(unused_variables)]
        let current_id = id();
        let p_id_uuid = Uuid::parse_str(&product_id.get()).unwrap_or_default();
        let c_id_uuid = Uuid::parse_str(&customer_id.get()).unwrap_or_default();

        let date_parsed = NaiveDate::parse_from_str(&date_of_sale.get(), "%Y-%m-%d")
            .unwrap_or_else(|_| Utc::now().date_naive())
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_local_timezone(Utc)
            .unwrap();

        #[allow(unused_variables)]
        let input = SaleInput {
            product_id: p_id_uuid,
            customer_id: c_id_uuid,
            date_of_sale: date_parsed,
            quantity: quantity.get(),
            discount: (discount.get() * 100.0) as i64,
            total_cents: (total.get() * 100.0) as i64,
            total_resolved: (resolved_amount.get() * 100.0) as i64,
            note: Some(note.get()),
        };
        
        #[allow(unused_mut)]
        let mut _navigate = navigate.clone();

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let req = if current_id == "create" || current_id.is_empty() {
                Request::post("/api/sales")
            } else {
                Request::put(&format!("/api/sales/{}", current_id))
            };

            if let Ok(_) = req
                .header("Authorization", &format!("Bearer {}", token))
                .json(&input).unwrap().send().await {
                 _navigate("/sales", Default::default());
            }
        });
    };

    view! {
        <div>
            <div style="margin-bottom: 2rem;">
                <A href="/sales" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Sales"</A>
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading); margin-top: 0.5rem;">
                    {move || if is_create() { "Record Sale" } else { "Edit Sale" }}
                </h1>
            </div>

            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); max-width: 600px;">
                <div style="display: flex; flex-direction: column; gap: 1.5rem;">
                    
                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Product"</label>
                            <select 
                                on:change=move |ev| set_product_id.set(event_target_value(&ev))
                                prop:value=product_id
                            >
                                <option value="">"Select Product..."</option>
                                <For
                                    each=move || products.get()
                                    key=|product| product.id
                                    children=move |product| view! {
                                        <option value=product.id.to_string()>{product.name}</option>
                                    }
                                />
                            </select>
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Customer"</label>
                            <select 
                                on:change=move |ev| set_customer_id.set(event_target_value(&ev))
                                prop:value=customer_id
                            >
                                <option value="">"Select Customer..."</option>
                                <For
                                    each=move || customers.get()
                                    key=|customer| customer.id
                                    children=move |customer| view! {
                                        <option value=customer.id.to_string()>{format!("{} {}", customer.first_name, customer.last_name)}</option>
                                    }
                                />
                            </select>
                        </div>
                    </div>

                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Date"</label>
                            <input 
                                type="date" 
                                prop:value=date_of_sale
                                on:input=move |ev| set_date_of_sale.set(event_target_value(&ev))
                            />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Quantity"</label>
                            <input 
                                type="number" 
                                prop:value=quantity
                                on:input=move |ev| set_quantity.set(event_target_value(&ev).parse().unwrap_or(0))
                            />
                        </div>
                    </div>

                     <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">{format!("Total Cost ({})", CURRENCY)}</label>
                            <input 
                                type="number" 
                                step="0.01"
                                prop:value=total
                                on:input=move |ev| set_total.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            />
                        </div>
                         <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">{format!("Discount ({})", CURRENCY)}</label>
                            <input 
                                type="number" 
                                step="0.01"
                                prop:value=discount
                                on:input=move |ev| set_discount.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            />
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">{format!("Amount Paid ({})", CURRENCY)}</label>
                            <input 
                                type="number" 
                                step="0.01"
                                prop:value=resolved_amount
                                on:input=move |ev| set_resolved_amount.set(event_target_value(&ev).parse().unwrap_or(0.0))
                            />
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Notes"</label>
                        <textarea 
                            prop:value=note
                            on:input=move |ev| set_note.set(event_target_value(&ev))
                            rows="3"
                        />
                    </div>

                    <button 
                        on:click=save_sale
                        class="btn-primary" 
                        style="margin-top: 1rem; padding: 0.75rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer;"
                    >
                        "Save Sale"
                    </button>
                </div>
            </div>
        </div>
    }
}
