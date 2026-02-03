use leptos::*;
use leptos_router::*;
use shared::models::{Product, ProductInput, ProductType, ProductDetailsInput};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn ProductListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (products, set_products) = create_signal(Vec::<Product>::new());
    
    let navigate = use_navigate();
    
    // Use Rc to share the fetch closure
    let fetch_products = std::rc::Rc::new({
        let navigate = navigate.clone();
        move || {
            let navigate = navigate.clone();
            #[cfg(target_arch = "wasm32")]
            spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                if let Ok(res) = Request::get("/api/products")
                    .header("Authorization", &format!("Bearer {}", token))
                    .send().await {
                    if res.status() == 401 {
                        navigate("/", Default::default());
                        return;
                    }
                    if let Ok(data) = res.json::<Vec<Product>>().await {
                        set_products.set(data);
                    }
                }
            });
        }
    });

    // Initial fetch
    let fetch_products_effect = fetch_products.clone();
    create_effect(move |_| {
        fetch_products_effect();
    });

    // Delete action also needs to be shared
    let fetch_products_delete = fetch_products.clone();
    let delete_action = std::rc::Rc::new(move |id: Uuid| {
        let fetch_products = fetch_products_delete.clone();
        #[allow(unused_variables)]
        let id = id;
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let _ = Request::delete(&format!("/api/products/{}", id))
                .header("Authorization", &format!("Bearer {}", token))
                .send()
                .await;
            
            // Refresh list
            fetch_products();
        });
        #[cfg(not(target_arch = "wasm32"))]
        {
             let _ = (id, fetch_products);
        }
    });

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Products"</h1>
                <A href="/products/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "Add Product"
                </A>
            </div>

            <div style="overflow-x: auto; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background-color: var(--bg-subtle); text-align: left;">
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Name"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Type"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Price"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Stock"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || products.get()
                            key=|product| product.id
                            children=move |product| {
                                let p_id = product.id;
                                let delete_handler = delete_action.clone();
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">{product.name}</td>
                                        <td style="padding: 1rem;">{product.product_type.as_str()}</td>
                                        <td style="padding: 1rem;">{format!("${:.2}", product.price_cents as f64 / 100.0)}</td>
                                        <td style="padding: 1rem;">{product.stock}</td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/products/{}", product.id) attr:style="color: var(--brand-primary); text-decoration: none; font-weight: 500;">"Edit"</A>
                                            <button 
                                                on:click=move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if web_sys::window().unwrap().confirm_with_message("Are you sure?").unwrap() {
                                                        delete_handler(p_id);
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
pub fn ProductEditPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();
    let is_create = move || id() == "create" || id().is_empty();

    let (name, set_name) = create_signal(String::new());
    let (description, set_description) = create_signal(String::new());
    let (price, set_price) = create_signal(0.0);
    let (stock, set_stock) = create_signal(0);
    let (prod_type, set_prod_type) = create_signal("physical_good".to_string());
    
    // Product Details State
    let (details, set_details) = create_signal(Vec::<ProductDetailsInput>::new());
    let (new_detail_name, set_new_detail_name) = create_signal(String::new());
    let (new_detail_value, set_new_detail_value) = create_signal(String::new());
    
    #[allow(unused_variables)]
    let navigate = use_navigate();

    // Load data if editing
    create_effect({
        let navigate = navigate.clone();
        move |_| {
            let current_id = id();
            let navigate = navigate.clone();
            if current_id != "create" && !current_id.is_empty() {
                #[cfg(target_arch = "wasm32")]
                spawn_local(async move {
                    let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                    if let Ok(res) = Request::get(&format!("/api/products/{}", current_id))
                        .header("Authorization", &format!("Bearer {}", token))
                        .send().await {
                        if res.status() == 401 {
                            navigate("/", Default::default());
                            return;
                        }
                        if let Ok(product) = res.json::<Product>().await {
                            set_name.set(product.name);
                            set_description.set(product.description);
                            set_price.set(product.price_cents as f64 / 100.0);
                            set_stock.set(product.stock);
                            set_prod_type.set(product.product_type.as_str().to_string());
                            
                            // Map ProductDetails to ProductDetailsInput
                            let mapped_details = product.details.into_iter().map(|d| ProductDetailsInput {
                                detail_name: d.detail_name,
                                detail_value: d.detail_value,
                            }).collect();
                            set_details.set(mapped_details);
                        }
                    }
                });
            }
        }
    });

    let add_detail = move |_| {
        let name = new_detail_name.get();
        let value = new_detail_value.get();
        if !name.is_empty() && !value.is_empty() {
            set_details.update(|d| d.push(ProductDetailsInput { detail_name: name, detail_value: value }));
            set_new_detail_name.set(String::new());
            set_new_detail_value.set(String::new());
        }
    };

    let remove_detail = move |index: usize| {
        set_details.update(|d| {
            if index < d.len() {
                d.remove(index);
            }
        });
    };

    let save_product = move |_| {
        #[allow(unused_variables)]
        let current_id = id();
        #[allow(unused_variables)]
        let input = ProductInput {
            name: name.get(),
            description: description.get(),
            price_cents: (price.get() * 100.0) as i64,
            stock: stock.get(),
            product_type: if prod_type.get() == "service" { ProductType::Service } else { ProductType::PhysicalGood },
            details: details.get(),
        };
        
        #[allow(unused_mut)]
        let mut navigate = navigate.clone();

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let req = if current_id == "create" || current_id.is_empty() {
                Request::post("/api/products")
            } else {
                Request::put(&format!("/api/products/{}", current_id))
            };

            if let Ok(_) = req
                .header("Authorization", &format!("Bearer {}", token))
                .json(&input).unwrap().send().await {
                 navigate("/products", Default::default());
            }
        });
    };

    view! {
        <div>
            <div style="margin-bottom: 2rem;">
                <A href="/products" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Products"</A>
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading); margin-top: 0.5rem;">
                    {move || if is_create() { "Create Product" } else { "Edit Product" }}
                </h1>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
                // Left Column: Product Information
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                    <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1.5rem; color: var(--text-heading);">"Product Information"</h2>
                    
                    <div style="display: flex; flex-direction: column; gap: 1.5rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                            <label style="font-weight: 500;">"Name"</label>
                            <input 
                                type="text" 
                                prop:value=name
                                on:input=move |ev| set_name.set(event_target_value(&ev))
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>

                        <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                            <label style="font-weight: 500;">"Description"</label>
                            <textarea 
                                prop:value=description
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                rows="4"
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>

                         <div style="display: flex; gap: 1rem;">
                            <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                                <label style="font-weight: 500;">"Price ($)"</label>
                                <input 
                                    type="number" 
                                    step="0.01"
                                    prop:value=price
                                    on:input=move |ev| set_price.set(event_target_value(&ev).parse().unwrap_or(0.0))
                                    style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                                />
                            </div>
                            <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                                <label style="font-weight: 500;">"Stock"</label>
                                <input 
                                    type="number" 
                                    prop:value=stock
                                    on:input=move |ev| set_stock.set(event_target_value(&ev).parse().unwrap_or(0))
                                    style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                                />
                            </div>
                        </div>

                        <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                            <label style="font-weight: 500;">"Type"</label>
                            <select 
                                on:change=move |ev| set_prod_type.set(event_target_value(&ev))
                                prop:value=prod_type
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            >
                                <option value="physical_good">"Physical Good"</option>
                                <option value="service">"Service"</option>
                            </select>
                        </div>
                    </div>
                </div>

                // Right Column: Product Details
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; flex-direction: column;">
                    <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1.5rem; color: var(--text-heading);">"Product Details"</h2>
                    
                    <div style="flex: 1; display: flex; flex-direction: column; gap: 1rem;">
                         <div style="display: flex; gap: 0.5rem; border-bottom: 1px solid var(--border-subtle); padding-bottom: 1rem; margin-bottom: 1rem;">
                            <input 
                                type="text" 
                                placeholder="Detail Name (e.g. Color)"
                                prop:value=new_detail_name
                                on:input=move |ev| set_new_detail_name.set(event_target_value(&ev))
                                style="flex: 1; padding: 0.5rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                            <input 
                                type="text" 
                                placeholder="Value (e.g. Red)"
                                prop:value=new_detail_value
                                on:input=move |ev| set_new_detail_value.set(event_target_value(&ev))
                                style="flex: 1; padding: 0.5rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                            <button 
                                on:click=add_detail
                                style="padding: 0.5rem 1rem; background-color: var(--brand-secondary); color: var(--brand-dark); border: none; border-radius: var(--radius-md); font-weight: 600; cursor: pointer;"
                            >
                                "Add"
                            </button>
                        </div>

                        <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                            <For
                                each=move || details.get().into_iter().enumerate()
                                key=|(i, _)| *i
                                children=move |(i, detail)| {
                                    let remove = move |_| remove_detail(i);
                                    view! {
                                        <div style="display: flex; justify-content: space-between; align-items: center; padding: 0.75rem; background: var(--bg-subtle); border-radius: var(--radius-md);">
                                            <div>
                                                <span style="font-weight: 600; margin-right: 0.5rem;">{detail.detail_name}:</span>
                                                <span>{detail.detail_value}</span>
                                            </div>
                                            <button 
                                                on:click=remove
                                                style="background: none; border: none; color: var(--text-muted); cursor: pointer; font-size: 1.25rem; line-height: 1;"
                                                title="Remove"
                                            >
                                                "×"
                                            </button>
                                        </div>
                                    }
                                }
                            />
                            <Show when=move || details.get().is_empty()>
                                <p style="text-align: center; color: var(--text-muted); font-style: italic; margin-top: 2rem;">"No details added yet."</p>
                            </Show>
                        </div>
                    </div>
                </div>
            </div>
            
            <div style="margin-top: 2rem; display: flex; justify-content: flex-end;">
                 <button 
                    on:click=save_product
                    class="btn-primary" 
                    style="padding: 1rem 2rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer; font-size: 1.1rem; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);"
                >
                    "Save Product"
                </button>
            </div>
        </div>
    }
}
