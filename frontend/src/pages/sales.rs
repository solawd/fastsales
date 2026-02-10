use leptos::*;
use leptos_router::*;
use shared::models::{Sale, SaleInput, SaleItem, SaleItemInput, SalesChannel, Product, Customer}; 
use uuid::Uuid;
use chrono::{Utc, NaiveDate, DateTime};
use crate::utils::CURRENCY;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SalesListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (sales, set_sales) = create_signal(Vec::<Sale>::new());
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    let (search_query, set_search_query) = create_signal(String::new());
    
    let navigate = use_navigate();
    
    let fetch_sales = move || {
        let _navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            let mut url = "/api/sales_transactions".to_string();
            let mut params = Vec::new();
            if !start_date.get().is_empty() {
                params.push(format!("start_date={}", start_date.get()));
            }
            if !end_date.get().is_empty() {
                params.push(format!("end_date={}", end_date.get()));
            }
            if !search_query.get().is_empty() {
                params.push(format!("query={}", search_query.get()));
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
                if let Ok(data) = res.json::<Vec<Sale>>().await {
                    set_sales.set(data);
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

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Sales Transactions"</h1>
                <A href="/sales/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "New Transaction"
                </A>
            </div>

            <div style="background: var(--bg-surface); padding: 1rem; border-radius: var(--radius-md); border: 1px solid var(--border-subtle); margin-bottom: 2rem;">
                <div style="display: flex; gap: 1rem; align-items: flex-end; flex-wrap: wrap;">
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1; min-width: 200px;">
                        <label style="font-weight: 500; font-size: 0.9rem;">"Search"</label>
                        <input 
                            type="text" 
                            placeholder="Customer Name or Receipt #"
                            prop:value=search_query
                            on:input=move |ev| set_search_query.set(event_target_value(&ev))
                        />
                    </div>
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
                </div>
            </div>

            <div style="overflow-x: auto; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background-color: var(--bg-subtle); text-align: left;">
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Date"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">{format!("Total ({})", CURRENCY)}</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Channel"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || sales.get()
                            key=|sale| sale.id
                            children=move |sale| {
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">{sale.date_and_time.format("%Y-%m-%d %H:%M").to_string()}</td>
                                        <td style="padding: 1rem;">{format!("{:.2}", sale.total_cents as f64 / 100.0)}</td>
                                        <td style="padding: 1rem;">{sale.sales_channel.to_string()}</td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/sales/{}", sale.id) attr:style="color: var(--brand-primary); text-decoration: none; font-weight: 500;">"View Details"</A>
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
pub fn SalesDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();
    
    let (sale, set_sale) = create_signal(None::<Sale>);
    let (customer_name, set_customer_name) = create_signal(String::new());

    create_effect(move |_| {
        let sale_id = id();
        if !sale_id.is_empty() {
             #[cfg(target_arch = "wasm32")]
             spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                if let Ok(res) = Request::get(&format!("/api/sales_transactions/{}", sale_id))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send().await {
                    if let Ok(data) = res.json::<Sale>().await {
                        // Fetch customer name if present
                        let c_id = data.customer_id;
                        set_sale.set(Some(data));

                        if let Some(cid) = c_id {
                             if let Ok(c_res) = Request::get(&format!("/api/customers/{}", cid))
                                .header("Authorization", &format!("Bearer {}", token))
                                .send().await {
                                    if let Ok(c_data) = c_res.json::<Customer>().await {
                                        set_customer_name.set(format!("{} {}", c_data.first_name, c_data.last_name));
                                    }
                             }
                        } else {
                            set_customer_name.set("Guest".to_string());
                        }
                    }
                }
             });
        }
    });

    view! {
        <div>
             <div style="margin-bottom: 2rem;">
                <A href="/sales" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Sales"</A>
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading); margin-top: 0.5rem;">"Sale Details"</h1>
            </div>

            {move || match sale.get() {
                Some(s) => view! {
                    <div style="display: flex; gap: 2rem; flex-wrap: wrap;">
                        // Left Pane: Sale Info
                        <div style="flex: 1; min-width: 300px; background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                            <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1rem;">"Transaction Info"</h2>
                            <div style="display: grid; gap: 1rem;">
                                <div><span style="color: var(--text-muted);">"Date:"</span> <span style="font-weight: 500; margin-left: 0.5rem;">{s.date_and_time.format("%Y-%m-%d %H:%M").to_string()}</span></div>
                                <div><span style="color: var(--text-muted);">"Customer:"</span> <span style="font-weight: 500; margin-left: 0.5rem;">{customer_name.get()}</span></div>
                                <div><span style="color: var(--text-muted);">"Channel:"</span> <span style="font-weight: 500; margin-left: 0.5rem;">{s.sales_channel.to_string()}</span></div>
                                <div><span style="color: var(--text-muted);">"Branch:"</span> <span style="font-weight: 500; margin-left: 0.5rem;">{s.company_branch}</span></div>
                                <div><span style="color: var(--text-muted);">"Receipt #:"</span> <span style="font-weight: 500; margin-left: 0.5rem;">{s.receipt_number}</span></div>
                                <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border-subtle);">
                                    <div style="font-size: 1.25rem; font-weight: 700;">"Total: " {format!("{} {:.2}", CURRENCY, s.total_cents as f64 / 100.0)}</div>
                                    <div style="font-size: 1rem; color: var(--text-muted);">"Resolved: " {format!("{} {:.2}", CURRENCY, s.total_resolved as f64 / 100.0)}</div>
                                </div>
                            </div>
                        </div>

                        // Right Pane: Items List
                        <div style="flex: 2; min-width: 400px; background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                            <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1rem;">"Items Purchased"</h2>
                            <table style="width: 100%; border-collapse: collapse;">
                                <thead>
                                    <tr style="text-align: left; border-bottom: 1px solid var(--border-subtle);">
                                        <th style="padding: 0.5rem;">"Item"</th>
                                        <th style="padding: 0.5rem;">"Qty"</th>
                                        <th style="padding: 0.5rem;">"Unit Price"</th>
                                        <th style="padding: 0.5rem;">"Total"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || s.sale_items.clone()
                                        key=|item| item.id
                                        children=move |item| view! {
                                            <tr style="border-bottom: 1px solid var(--border-subtle);">
                                                <td style="padding: 0.75rem 0.5rem;">{item.product_name.clone().unwrap_or(item.product_id.to_string())}</td> // Should fetch product name ideally
                                                <td style="padding: 0.75rem 0.5rem;">{item.quantity}</td>
                                                <td style="padding: 0.75rem 0.5rem;">
                                                    {move || {
                                                        let cents = item.price_per_item.unwrap_or_else(|| {
                                                            if item.quantity > 0 { item.total_cents / item.quantity } else { 0 }
                                                        });
                                                        format!("{:.2}", cents as f64 / 100.0)
                                                    }}
                                                </td>
                                                <td style="padding: 0.75rem 0.5rem;">{format!("{:.2}", item.total_cents as f64 / 100.0)}</td>
                                            </tr>
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    </div>
                }.into_view(),
                None => view! { <div>"Loading..."</div> }.into_view()
            }}
        </div>
    }
}

#[component]
pub fn SalesCreatePage() -> impl IntoView {
    // State
    let (customer_id, set_customer_id) = create_signal(String::new());
    let (channel, set_channel) = create_signal("mobile".to_string());
    let (branch, set_branch) = create_signal("Main Branch".to_string());
    
    // Items List
    // We need a struct to hold temporary item state before creating the final SaleItemInput
    #[derive(Clone, Debug, PartialEq)]
    struct TempItem {
        id: Uuid, // temp id for key
        product_id: String,
        quantity: i64,
        unit_price: f64,
        discount: f64,
    }

    let (items, set_items) = create_signal(vec![TempItem { 
        id: Uuid::new_v4(), 
        product_id: "".to_string(), 
        quantity: 1, 
        unit_price: 0.0, 
        discount: 0.0 
    }]);

    // Data lists
    let (products, set_products) = create_signal(Vec::<Product>::new());
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    
    create_effect(move |_| {
         #[cfg(target_arch = "wasm32")]
         spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            // Fetch Products
            if let Ok(res) = Request::get("/api/products").header("Authorization", &format!("Bearer {}", token)).send().await {
                if let Ok(data) = res.json::<Vec<Product>>().await {
                    set_products.set(data);
                }
            }
            // Fetch Customers
            if let Ok(res) = Request::get("/api/customers").header("Authorization", &format!("Bearer {}", token)).send().await {
                if let Ok(data) = res.json::<Vec<Customer>>().await {
                    set_customers.set(data);
                }
            }
         });
    });

    let add_item = move |_| {
        set_items.update(|list| list.push(TempItem {
            id: Uuid::new_v4(),
            product_id: "".to_string(),
            quantity: 1,
            unit_price: 0.0,
            discount: 0.0,
        }));
    };

    let remove_item = move |id: Uuid| {
        set_items.update(|list| list.retain(|item| item.id != id));
    };

    let update_item_product = move |id: Uuid, p_id: String| {
        // Also auto-set price if possible
        let p_list = products.get();
        let price = p_list.iter().find(|p| p.id.to_string() == p_id).map(|p| p.price_cents as f64 / 100.0).unwrap_or(0.0);
        
        set_items.update(|list| {
            if let Some(item) = list.iter_mut().find(|i| i.id == id) {
                item.product_id = p_id;
                item.unit_price = price;
            }
        });
    };

    let update_item_qty = move |id: Uuid, qty: i64| {
        set_items.update(|list| {
             if let Some(item) = list.iter_mut().find(|i| i.id == id) {
                item.quantity = qty;
             }
        });
    };

    let calculate_total = move || {
        items.get().iter().fold(0.0, |acc, item| {
            acc + (item.quantity as f64 * item.unit_price) - item.discount
        })
    };

    let navigate = use_navigate();
    let save_transaction = move |_| {
        let current_items = items.get();
        let total_val = calculate_total();
        
        // Prepare SaleInput
        let sale_items: Vec<SaleItemInput> = current_items.iter().map(|item| {
            let total_cents = (item.quantity as f64 * item.unit_price * 100.0) as i64;
            SaleItemInput {
                sale_id: None, // Will be set by backend
                product_id: Uuid::parse_str(&item.product_id).unwrap_or_default(),
                customer_id: Uuid::parse_str(&customer_id.get()).unwrap_or_default(), // Items linked to customer too for legacy?
                date_of_sale: Utc::now(), // Use transaction time
                quantity: item.quantity,
                discount: (item.discount * 100.0) as i64,
                total_cents,
                total_resolved: total_cents - (item.discount * 100.0) as i64, 
                note: None,
            }
        }).collect();

        let input = SaleInput {
            customer_id: if customer_id.get().is_empty() { None } else { Some(Uuid::parse_str(&customer_id.get()).unwrap_or_default()) },
            date_and_time: Utc::now(),
            sale_items,
            total_cents: (total_val * 100.0) as i64,
            discount: 0, // Global discount not implemented yet
            total_resolved: (total_val * 100.0) as i64,
            sales_channel: if channel.get() == "mobile" { SalesChannel::Mobile } else { SalesChannel::Web },
            staff_responsible: Uuid::nil(), // TODO: Get from auth context?
            company_branch: branch.get(),
            car_number: "".to_string(),
            receipt_number: Uuid::new_v4().to_string().chars().take(8).collect(),
        };

        let _navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let res = Request::post("/api/sales_transactions")
                .header("Authorization", &format!("Bearer {}", token))
                .json(&input).unwrap()
                .send().await;
            
            if let Ok(r) = res {
                if r.ok() {
                    _navigate("/sales", Default::default());
                } else {
                    // Show error?
                }
            }
        });
    };

    view! {
        <div>
             <div style="margin-bottom: 2rem;">
                <A href="/sales" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Sales"</A>
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading); margin-top: 0.5rem;">"New Sales Transaction"</h1>
            </div>

            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); max-width: 800px;">
                // Header Info
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1rem; margin-bottom: 2rem;">
                     <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Customer"</label>
                        <select 
                            on:change=move |ev| set_customer_id.set(event_target_value(&ev))
                            prop:value=customer_id
                        >
                            <option value="">"Select Customer..."</option>
                            <For
                                each=move || customers.get()
                                key=|c| c.id
                                children=move |c| view! { <option value=c.id.to_string()>{format!("{} {}", c.first_name, c.last_name)}</option> }
                            />
                        </select>
                    </div>
                     <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Channel"</label>
                        <select 
                            on:change=move |ev| set_channel.set(event_target_value(&ev))
                            prop:value=channel
                        >
                            <option value="mobile">"Mobile"</option>
                            <option value="web">"Web"</option>
                        </select>
                    </div>
                </div>

                // Items
                <h3 style="font-size: 1.1rem; font-weight: 600; margin-bottom: 1rem;">"Items"</h3>
                <div style="display: flex; flex-direction: column; gap: 1rem;">
                    <For
                        each=move || items.get()
                        key=|item| item.id
                        children=move |item| {
                            let i_id = item.id;
                            let _remove = remove_item.clone();
                            let _update_prod = update_item_product.clone();
                            let _update_qty = update_item_qty.clone();
                            view! {
                                <div style="display: flex; gap: 1rem; align-items: flex-end; padding: 1rem; background: var(--bg-subtle); border-radius: var(--radius-md);">
                                    <div style="flex: 2; display: flex; flex-direction: column; gap: 0.25rem;">
                                        <label style="font-size: 0.85rem;">"Product"</label>
                                        <select
                                            on:change=move |ev| _update_prod(i_id, event_target_value(&ev))
                                            prop:value=item.product_id
                                        >
                                            <option value="">"Select..."</option>
                                            <For
                                                each=move || products.get()
                                                key=|p| p.id
                                                children=move |p| view! { <option value=p.id.to_string()>{p.name}</option> }
                                            />
                                        </select>
                                    </div>
                                    <div style="flex: 1; display: flex; flex-direction: column; gap: 0.25rem;">
                                        <label style="font-size: 0.85rem;">"Qty"</label>
                                        <input 
                                            type="number" min="1"
                                            prop:value=item.quantity
                                            on:input=move |ev| _update_qty(i_id, event_target_value(&ev).parse().unwrap_or(1))
                                        />
                                    </div>
                                    <div style="flex: 1; display: flex; flex-direction: column; gap: 0.25rem;">
                                        <label style="font-size: 0.85rem;">"Price"</label>
                                        <div style="padding: 0.5rem;">{format!("{:.2}", item.unit_price)}</div>
                                    </div>
                                     <div style="flex: 1; display: flex; flex-direction: column; gap: 0.25rem;">
                                        <label style="font-size: 0.85rem;">"Subtotal"</label>
                                        <div style="padding: 0.5rem; font-weight: 600;">{format!("{:.2}", item.quantity as f64 * item.unit_price)}</div>
                                    </div>
                                    <button 
                                        on:click=move |_| _remove(i_id)
                                        style="color: var(--state-error); background: none; border: none; cursor: pointer; padding: 0.5rem;"
                                    >
                                        "✕"
                                    </button>
                                </div>
                            }
                        }
                    />
                </div>
                
                 <button 
                    on:click=add_item
                    style="margin-top: 1rem; font-size: 0.9rem; color: var(--brand-primary); background: none; border: 1px dashed var(--brand-primary); padding: 0.5rem 1rem; border-radius: var(--radius-md); cursor: pointer;"
                >
                    "+ Add Item"
                </button>

                <div style="margin-top: 2rem; border-top: 1px solid var(--border-subtle); padding-top: 1rem; text-align: right;">
                    <div style="font-size: 1.5rem; font-weight: 700;">
                        "Total: " {move || format!("{} {:.2}", CURRENCY, calculate_total())}
                    </div>
                     <button 
                        on:click=save_transaction
                        class="btn-primary" 
                        style="margin-top: 1rem; padding: 0.75rem 2rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer;"
                    >
                        "Complete Transaction"
                    </button>
                </div>
            </div>
        </div>
    }
}
