use leptos::*;
use leptos_router::*;
use shared::models::{Customer, CustomerInput};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn CustomersListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (customers, set_customers) = create_signal(Vec::<Customer>::new());
    
    let navigate = use_navigate();

    let fetch_customers = move || {
        let navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            if let Ok(res) = Request::get("/api/customers")
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                
                if res.status() == 401 {
                    navigate("/", Default::default());
                    return;
                }

                if let Ok(data) = res.json::<Vec<Customer>>().await {
                    set_customers.set(data);
                }
            }
        });
    };

    create_effect({
        let fetch_customers = fetch_customers.clone();
        move |_| {
            fetch_customers();
        }
    });

    let delete_action = move |id: Uuid| {
        #[allow(unused_variables)]
        let id = id;
        let fetch_customers = fetch_customers.clone(); // Clone for async block
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            let _ = Request::delete(&format!("/api/customers/{}", id))
                .header("Authorization", &format!("Bearer {}", token))
                .send()
                .await;
            fetch_customers();
        });
    };

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Customers"</h1>
                <A href="/customers/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "Add Customer"
                </A>
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
                                let c_id = customer.id;
                                let delete_handler = delete_action.clone();
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">{format!("{} {}", customer.first_name, customer.last_name)}</td>
                                        <td style="padding: 1rem;">{customer.email}</td>
                                        <td style="padding: 1rem;">{customer.mobile_number}</td>
                                        <td style="padding: 1rem;">{customer.date_of_birth}</td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/customers/{}", customer.id) attr:style="color: var(--brand-primary); text-decoration: none; font-weight: 500;">"Edit"</A>
                                            <button 
                                                on:click=move |_| {
                                                    #[cfg(target_arch = "wasm32")]
                                                    if web_sys::window().unwrap().confirm_with_message("Are you sure?").unwrap() {
                                                        delete_handler(c_id);
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

    #[allow(unused_variables)]
    let navigate = use_navigate();

    // Load data if editing
    create_effect({
        let navigate = navigate.clone();
        move |_| {
            let current_id = id();
            let navigate = navigate.clone(); // Clone for async block
            if current_id != "create" && !current_id.is_empty() {
                #[cfg(target_arch = "wasm32")]
                spawn_local(async move {
                    let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                    if let Ok(res) = Request::get(&format!("/api/customers/{}", current_id))
                        .header("Authorization", &format!("Bearer {}", token))
                        .send().await {
                        
                        if res.status() == 401 {
                            navigate("/", Default::default());
                            return;
                        }

                        if let Ok(customer) = res.json::<Customer>().await {
                            set_first_name.set(customer.first_name);
                            set_last_name.set(customer.last_name);
                            set_middle_name.set(customer.middle_name.unwrap_or_default());
                            set_email.set(customer.email);
                            set_mobile_number.set(customer.mobile_number);
                            set_date_of_birth.set(customer.date_of_birth);
                        }
                    }
                });
            }
        }
    });

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
            date_of_birth: date_of_birth.get(),
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

            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); max-width: 600px;">
                <div style="display: flex; flex-direction: column; gap: 1.5rem;">
                    
                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"First Name"</label>
                            <input 
                                type="text" 
                                prop:value=first_name
                                on:input=move |ev| set_first_name.set(event_target_value(&ev))
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Last Name"</label>
                            <input 
                                type="text" 
                                prop:value=last_name
                                on:input=move |ev| set_last_name.set(event_target_value(&ev))
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Middle Name (Optional)"</label>
                        <input 
                            type="text" 
                            prop:value=middle_name
                            on:input=move |ev| set_middle_name.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>

                    <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                        <label style="font-weight: 500;">"Email"</label>
                        <input 
                            type="email" 
                            prop:value=email
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>

                    <div style="display: flex; gap: 1rem;">
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Mobile Number"</label>
                            <input 
                                type="tel" 
                                prop:value=mobile_number
                                on:input=move |ev| set_mobile_number.set(event_target_value(&ev))
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>
                        <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                            <label style="font-weight: 500;">"Date of Birth (DD-MM-YYYY)"</label>
                            <input 
                                type="text"
                                placeholder="DD-MM-YYYY" 
                                prop:value=date_of_birth
                                on:input=move |ev| set_date_of_birth.set(event_target_value(&ev))
                                style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                            />
                        </div>
                    </div>

                    <button 
                        on:click=save_customer
                        class="btn-primary" 
                        style="margin-top: 1rem; padding: 0.75rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600; border: none; cursor: pointer;"
                    >
                        "Save Customer"
                    </button>
                </div>
            </div>
        </div>
    }
}
