use leptos::*;
use leptos_router::*;
use shared::models::{Customer, CustomerInput, CustomerDetailsInput};
use uuid::Uuid;
use chrono::NaiveDate;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn CustomersListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    let (search_query, set_search_query) = create_signal(String::new());
    
    let navigate = use_navigate();

    let _fetch_customers = move || {
        let _navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            let mut url = "/api/customers".to_string();
            if !search_query.get().is_empty() {
                url.push_str(&format!("?search={}", search_query.get()));
            }

            if let Ok(res) = Request::get(&url)
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                
                if res.status() == 401 {
                    _navigate("/", Default::default());
                    return;
                }

                if let Ok(data) = res.json::<Vec<Customer>>().await {
                    set_customers.set(data);
                }
            }
        });
    };

    create_effect({
        let _fetch_customers = _fetch_customers.clone();
        move |_| {
            _fetch_customers();
        }
    });

    let delete_action = {
        let _fetch_customers = _fetch_customers.clone();
        move |id: Uuid| {
            let _fetch_customers = _fetch_customers.clone();
            #[allow(unused_variables)]
            let id = id;
            #[cfg(target_arch = "wasm32")]
            spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                let _ = Request::delete(&format!("/api/customers/{}", id))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send()
                    .await;
                _fetch_customers();
            });
        }
    };

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Customers"</h1>
                <A href="/customers/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "Add Customer"
                </A>
            </div>

            <div style="margin-bottom: 2rem; display: flex; gap: 1rem;">
                 <input 
                    type="text" 
                    placeholder="Search customers..."
                    prop:value=search_query
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    on:keydown={
                        let _fetch_customers = _fetch_customers.clone();
                        move |ev| {
                            if ev.key() == "Enter" {
                                _fetch_customers();
                            }
                        }
                    }
                    style="width: 40%;"
                />
                <button 
                    class="btn-primary"
                    on:click={
                        let _fetch_customers = _fetch_customers.clone();
                        move |_| _fetch_customers()
                    }
                    style="padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); border: none; font-weight: 600; cursor: pointer;"
                >
                    "Search"
                </button>
            </div>

            <div style="overflow-x: auto; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background-color: var(--bg-subtle); text-align: left;">
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Name"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Email"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Mobile"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"DOB"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || customers.get()
                            key=|customer| customer.id
                            children=move |customer| {
                                let _c_id = customer.id;
                                let _delete_handler = delete_action.clone();
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">{format!("{} {}", customer.first_name, customer.last_name)}</td>
                                        <td style="padding: 1rem;">{customer.email}</td>
                                        <td style="padding: 1rem;">{customer.mobile_number}</td>
                                        <td style="padding: 1rem;">{customer.date_of_birth.format("%d-%m-%Y").to_string()}</td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/customers/{}", customer.id) attr:style="color: var(--brand-primary); text-decoration: none; font-weight: 500;">"Edit"</A>
                                            <button 
                                                on:click=move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if web_sys::window().unwrap().confirm_with_message("Are you sure?").unwrap() {
                                                        _delete_handler(_c_id);
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
pub fn CustomerEditPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();
    let is_create = move || id() == "create" || id().is_empty();

    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (middle_name, set_middle_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (mobile_number, set_mobile_number) = create_signal(String::new());
    let (date_of_birth, set_date_of_birth) = create_signal(String::new());
    
    // Details State
    let (details, set_details) = create_signal(Vec::<CustomerDetailsInput>::new());
    let (new_detail_name, set_new_detail_name) = create_signal(String::new());
    let (new_detail_value, set_new_detail_value) = create_signal(String::new());

    #[allow(unused_variables)]
    let navigate = use_navigate();

    // Load data if editing
    create_effect({
        let navigate = navigate.clone();
        move |_| {
            let current_id = id();
            let _navigate = navigate.clone();
            if current_id != "create" && !current_id.is_empty() {
                #[cfg(target_arch = "wasm32")]
                spawn_local(async move {
                    let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                    if let Ok(res) = Request::get(&format!("/api/customers/{}", current_id))
                        .header("Authorization", &format!("Bearer {}", token))
                        .send().await {
                        
                        if res.status() == 401 {
                            _navigate("/", Default::default());
                            return;
                        }

                        if let Ok(customer) = res.json::<Customer>().await {
                            set_first_name.set(customer.first_name);
                            set_last_name.set(customer.last_name);
                            set_middle_name.set(customer.middle_name.unwrap_or_default());
                            set_email.set(customer.email);
                            set_mobile_number.set(customer.mobile_number);
                            set_date_of_birth.set(customer.date_of_birth.to_string());
                            
                            // Map Details
                            let mapped_details = customer.details.into_iter().map(|d| CustomerDetailsInput {
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
            set_details.update(|d| d.push(CustomerDetailsInput { detail_name: name, detail_value: value }));
            set_new_detail_name.set(String::new());
            set_new_detail_value.set(String::new());
        }
    };



    let save_customer = move |_| {
        #[allow(unused_variables)]
        let current_id = id();
        #[allow(unused_variables)]
        let input = CustomerInput {
            first_name: first_name.get(),
            last_name: last_name.get(),
            middle_name: {
                let m = middle_name.get();
                if m.is_empty() { None } else { Some(m) }
            },
            email: email.get(),
            mobile_number: mobile_number.get(),
            date_of_birth: NaiveDate::parse_from_str(&date_of_birth.get(), "%Y-%m-%d").unwrap_or_default(),
            details: details.get(),
        };
        
        #[allow(unused_mut)]
        let mut _navigate = navigate.clone();

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let req = if current_id == "create" || current_id.is_empty() {
                Request::post("/api/customers")
            } else {
                Request::put(&format!("/api/customers/{}", current_id))
            };

            if let Ok(_) = req
                .header("Authorization", &format!("Bearer {}", token))
                .json(&input).unwrap().send().await {
                 _navigate("/customers", Default::default());
            }
        });
    };

    view! {
        <div>
            <div style="margin-bottom: 2rem;">
                <A href="/customers" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Customers"</A>
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading); margin-top: 0.5rem;">
                    {move || if is_create() { "Create Customer" } else { "Edit Customer" }}
                </h1>
            </div>

            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 2rem;">
                // Left Column: Customer Information
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                     <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1.5rem; color: var(--text-heading);">"Customer Information"</h2>
                     <div style="display: flex; flex-direction: column; gap: 1.5rem;">
                    
                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"First Name"</label>
                            <input 
                                type="text" 
                                prop:value=first_name
                                on:input=move |ev| set_first_name.set(event_target_value(&ev))
                            />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Last Name"</label>
                            <input 
                                type="text" 
                                prop:value=last_name
                                on:input=move |ev| set_last_name.set(event_target_value(&ev))
                            />
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Middle Name (Optional)"</label>
                        <input 
                            type="text" 
                            prop:value=middle_name
                            on:input=move |ev| set_middle_name.set(event_target_value(&ev))
                        />
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Email"</label>
                        <input 
                            type="email" 
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                        />
                    </div>

                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Mobile Number"</label>
                            <input 
                                type="tel" 
                                prop:value=mobile_number
                                on:input=move |ev| set_mobile_number.set(event_target_value(&ev))
                            />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Date of Birth"</label>
                            <input 
                                type="date"
                                prop:value=date_of_birth
                                on:input=move |ev| set_date_of_birth.set(event_target_value(&ev))
                            />
                        </div>
                    </div>
                    </div>
                </div>
                
                 // Right Column: Customer Details
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; flex-direction: column;">
                    <h2 style="font-size: 1.25rem; font-weight: 600; margin-bottom: 1.5rem; color: var(--text-heading);">"Additional Details"</h2>
                    
                    <div style="display: flex; flex-direction: column; gap: 1rem; flex: 1;">
                        <div style="display: flex; gap: 0.5rem;">
                            <input
                                type="text"
                                placeholder="Detail Name (e.g. Loyalty ID)"
                                prop:value=new_detail_name
                                on:input=move |ev| set_new_detail_name.set(event_target_value(&ev))
                                style="flex: 1;"
                            />
                            <input
                                type="text"
                                placeholder="Value"
                                prop:value=new_detail_value
                                on:input=move |ev| set_new_detail_value.set(event_target_value(&ev))
                                style="flex: 1;"
                            />
                            <button
                                on:click=add_detail
                                class="btn-primary"
                                style="padding: 0.75rem 1rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer;"
                            >
                                "Add"
                            </button>
                        </div>
                        
                        <div style="margin-top: 1rem; display: flex; flex-direction: column; gap: 0.5rem;">
                             <For
                                each=move || details.get()
                                key=|detail| (detail.detail_name.clone(), detail.detail_value.clone())
                                children=move |detail| {
                                    // Need to capture index for deletion, but For uses keyed items. 
                                    // A simple way is to iterate with index in the view or finding index.
                                    // NOTE: Because removing by index from a signal inside For is tricky without index signal,
                                    // We can just rely on state refresh or use a better key.
                                    // For simplicity, we'll iterate the list to find index or just filter.
                                    // Actually, let's use a computed list with indices if needed, but for now simple view:
                                    let d_name = detail.detail_name.clone();
                                    
                                    view! {
                                        <div style="display: flex; align-items: center; justify-content: space-between; padding: 0.75rem; background: var(--bg-subtle); border-radius: var(--radius-md);">
                                            <div>
                                                <span style="font-weight: 600; margin-right: 0.5rem;">{detail.detail_name}:</span>
                                                <span>{detail.detail_value}</span>
                                            </div>
                                            <button
                                                on:click=move |_| {
                                                    // Find index
                                                    set_details.update(|d| {
                                                        if let Some(pos) = d.iter().position(|x| x.detail_name == d_name) {
                                                            d.remove(pos);
                                                        }
                                                    });
                                                }
                                                style="background: none; border: none; color: var(--state-error); cursor: pointer; font-size: 1.2rem; line-height: 1;"
                                                title="Remove"
                                            >
                                                "×"
                                            </button>
                                        </div>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </div>
            
            <div style="margin-top: 2rem; display: flex; justify-content: flex-end;">
                 <button 
                    on:click=save_customer
                    class="btn-primary" 
                    style="padding: 1rem 2rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer; font-size: 1rem;"
                >
                    "Save Customer"
                </button>
            </div>
        </div>
    }
}
