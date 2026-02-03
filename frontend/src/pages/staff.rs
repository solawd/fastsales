use leptos::*;
use leptos_router::*;
use shared::models::{Staff, StaffInput};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use web_sys::Event;

#[component]
pub fn StaffListPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (staff_list, set_staff_list) = create_signal(Vec::<Staff>::new());

    let navigate = use_navigate();

    create_effect(move |_| {
        #[allow(unused_variables)]
        let navigate = navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
            if token.is_empty() {
                navigate("/", Default::default());
                return;
            }
            if let Ok(res) = Request::get("/api/staff").header("Authorization", &format!("Bearer {}", token)).send().await {
                 if res.status() == 401 {
                     navigate("/", Default::default());
                     return;
                 }
                 if let Ok(list) = res.json::<Vec<Staff>>().await {
                     set_staff_list.set(list);
                 }
            }
        });
    });

    let delete_staff = move |id: String| {
        #[allow(unused_variables)]
        let id = id;
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
            if Request::delete(&format!("/api/staff/{}", id)).header("Authorization", &format!("Bearer {}", token)).send().await.is_ok() {
                set_staff_list.update(|list| list.retain(|s| s.id.to_string() != id));
            }
        });
    };

    view! {
        <div>
            <div style="width: 100%; display: flex; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Staff"</h1>
                <A href="/staff/create" class="btn-primary" attr:style="margin-left: auto; text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border-radius: var(--radius-md); font-weight: 600;">
                    "Add Staff"
                </A>
            </div>

            <div style="overflow-x: auto; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="background-color: var(--bg-subtle); text-align: left;">
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Name"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Staff ID"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Username"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Role"</th>
                            <th style="padding: 1rem; border-bottom: 1px solid var(--border-subtle);">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <For
                            each=move || staff_list.get()
                            key=|staff| staff.id
                            children=move |staff| {
                                let delete_action = delete_staff.clone();
                                view! {
                                    <tr style="border-bottom: 1px solid var(--border-subtle);">
                                        <td style="padding: 1rem;">
                                            <div style="display: flex; align-items: center; gap: 0.75rem;">
                                                <img 
                                                    src=if staff.photo_link.is_empty() { "https://ui-avatars.com/api/?name=".to_string() + &staff.first_name + "+" + &staff.last_name } else { staff.photo_link.clone() }
                                                    alt="Avatar" 
                                                    style="width: 32px; height: 32px; border-radius: 50%; object-fit: cover;"
                                                />
                                                <span>{format!("{} {}", staff.first_name, staff.last_name)}</span>
                                            </div>
                                        </td>
                                        <td style="padding: 1rem;">{staff.staff_id}</td>
                                        <td style="padding: 1rem;">{staff.username}</td>
                                        <td style="padding: 1rem;">"Staff"</td>
                                        <td style="padding: 1rem; display: flex; gap: 0.5rem;">
                                            <A href=format!("/staff/{}", staff.id) attr:style="text-decoration: none; color: var(--brand-primary); font-weight: 600;">"Edit"</A>
                                            <button 
                                                on:click=move |_| delete_action(staff.id.to_string())
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
pub fn StaffEditPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();
    let is_create = move || id() == "create" || id().is_empty();

    let (staff_id_field, set_staff_id_field) = create_signal(String::new());
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (mobile_number, set_mobile_number) = create_signal(String::new());
    let (photo_link, set_photo_link) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let navigate = use_navigate();
    
    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        #[allow(unused_variables)]
        let navigate = navigate_effect.clone();
        #[allow(unused_variables)]
        let current_id = id();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
             let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
             if token.is_empty() {
                 navigate("/", Default::default());
                 return;
             }
            if current_id != "create" && !current_id.is_empty() {
                if let Ok(res) = Request::get(&format!("/api/staff/{}", current_id)).header("Authorization", &format!("Bearer {}", token)).send().await {
                    if res.status() == 401 {
                        navigate("/", Default::default());
                        return;
                    }
                    if let Ok(staff) = res.json::<Staff>().await {
                        set_staff_id_field.set(staff.staff_id);
                        set_first_name.set(staff.first_name);
                        set_last_name.set(staff.last_name);
                        set_mobile_number.set(staff.mobile_number);
                        set_photo_link.set(staff.photo_link);
                        set_username.set(staff.username);
                    }
                }
            }
        });
    });

    let save_staff = move |_| {
        #[allow(unused_variables)]
        let navigate = navigate.clone();
        #[allow(unused_variables)]
        let current_id = id();
        #[allow(unused_variables)]
        let input = StaffInput {
            id: if current_id == "create" || current_id.is_empty() { None } else { Uuid::parse_str(&current_id).ok() },
            staff_id: staff_id_field.get(),
            first_name: first_name.get(),
            last_name: last_name.get(),
            mobile_number: mobile_number.get(),
            photo_link: photo_link.get(),
            username: username.get(),
            password: if password.get().is_empty() { None } else { Some(password.get()) },
        };

        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
            
            if token.is_empty() {
                navigate("/", Default::default());
                return;
            }

            let req = if current_id == "create" || current_id.is_empty() {
                Request::post("/api/staff")
            } else {
                Request::put(&format!("/api/staff/{}", current_id))
            };

            if let Ok(res) = req.header("Authorization", &format!("Bearer {}", token))
                .header("Content-Type", "application/json")
                .body(serde_json::to_string(&input).unwrap())
                .unwrap()
                .send()
                .await 
            {
                if res.status() == 401 {
                    navigate("/", Default::default());
                    return;
                }
                if res.ok() {
                    navigate("/staff", Default::default());
                }
            }
        });
    };

    #[cfg(target_arch = "wasm32")]
    fn event_target_value(e: &Event) -> String {
        e.target().expect("target").dyn_into::<web_sys::HtmlInputElement>().expect("input element").value()
    }

    view! {
        <div style="max-width: 800px; margin: 0 auto;">
            <div style="display: flex; justify_content: space-between; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">
                    {move || if is_create() { "Add Staff" } else { "Edit Staff" }}
                </h1>
            </div>

            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 1.5rem;">
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

                <div style="display: flex; gap: 1rem;">
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Staff ID (Unique)"</label>
                        <input 
                            type="text" 
                            prop:value=staff_id_field
                            on:input=move |ev| set_staff_id_field.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Mobile Number"</label>
                        <input 
                            type="text" 
                            prop:value=mobile_number
                            on:input=move |ev| set_mobile_number.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>
                </div>

                <div style="display: flex; gap: 1rem;">
                     <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Username"</label>
                        <input 
                            type="text" 
                            prop:value=username
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Password"</label>
                        <input 
                            type="password" 
                            placeholder=move || if is_create() { "Required" } else { "Leave blank to keep current" }
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>
                </div>
                
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <label style="font-weight: 500;">"Photo URL"</label>
                    <input 
                        type="text" 
                        prop:value=photo_link
                        on:input=move |ev| set_photo_link.set(event_target_value(&ev))
                        style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                    />
                </div>

                <div style="display: flex; justify-content: flex-end; gap: 1rem; margin-top: 1rem;">
                    <A href="/staff" class="btn-secondary" attr:style="text-decoration: none; display: inline-block; padding: 0.75rem 1.5rem; background-color: var(--bg-subtle); color: var(--text-body); border-radius: var(--radius-md); font-weight: 600;">
                        "Cancel"
                    </A>
                    <button 
                        on:click=save_staff
                        style="padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border: none; border-radius: var(--radius-md); font-weight: 600; cursor: pointer;"
                    >
                        "Save Staff"
                    </button>
                </div>
            </div>
        </div>
    }
}
