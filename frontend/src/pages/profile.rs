use leptos::*;
use leptos_router::*;
use shared::models::{Staff, StaffInput, UploadResponse};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::JsFuture;
use web_sys::{FormData, RequestInit, Response};
use web_sys::Event;

#[component]
pub fn ProfilePage() -> impl IntoView {
    let (staff_uuid, set_staff_uuid) = create_signal(String::new());
    let (staff_id_field, set_staff_id_field) = create_signal(String::new());
    let (first_name, set_first_name) = create_signal(String::new());
    let (last_name, set_last_name) = create_signal(String::new());
    let (mobile_number, set_mobile_number) = create_signal(String::new());
    let (photo_link, set_photo_link) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let navigate = use_navigate();
    
    // Fetch current user profile on load
    let navigate_effect = navigate.clone();
    create_effect(move |_| {
        #[allow(unused_variables)]
        let navigate = navigate_effect.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
             let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
             if token.is_empty() {
                 navigate("/", Default::default());
                 return;
             }
            
            if let Ok(res) = Request::get("/api/auth/profile").header("Authorization", &format!("Bearer {}", token)).send().await {
                if res.status() == 401 {
                    navigate("/", Default::default());
                    return;
                }
                if let Ok(staff) = res.json::<Staff>().await {
                    set_staff_uuid.set(staff.id.to_string());
                    set_staff_id_field.set(staff.staff_id);
                    set_first_name.set(staff.first_name);
                    set_last_name.set(staff.last_name);
                    set_mobile_number.set(staff.mobile_number);
                    set_photo_link.set(staff.photo_link);
                    set_username.set(staff.username);
                }
            }
        });
    });

    let save_profile = move |_| {
        #[allow(unused_variables)]
        let navigate = navigate.clone();
        #[allow(unused_variables)]
        let current_uuid = staff_uuid.get();
        
        if current_uuid.is_empty() {
            return;
        }

        #[allow(unused_variables)]
        let input = StaffInput {
            id: Uuid::parse_str(&current_uuid).ok(),
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

            // Update using the staff ID (UUID) we fetched
            if let Ok(res) = Request::put(&format!("/api/staff/{}", current_uuid))
                .header("Authorization", &format!("Bearer {}", token))
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
                    // Stay on profile page or show success message?
                    // For now, let's just clear the password field to indicate success/reset state
                    set_password.set(String::new());
                    // Ideally we might want a toast notification
                }
            }
        });
    };

    #[cfg(target_arch = "wasm32")]
    let upload_photo = move |ev: Event| {
        let target = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        if let Some(files) = target.files() {
            if let Some(file) = files.get(0) {
                spawn_local(async move {
                     let form_data = FormData::new().unwrap();
                     form_data.append_with_blob("file", &file).unwrap();

                     let token = window().local_storage().ok().flatten().and_then(|s| s.get_item("jwt_token").ok().flatten()).unwrap_or_default();
                     
                     let mut opts = RequestInit::new();
                     opts.set_method("POST");
                     opts.set_body(&form_data);
                     
                     let request = web_sys::Request::new_with_str_and_init("/api/upload", &opts).unwrap();
                     request.headers().set("Authorization", &format!("Bearer {}", token)).unwrap();
                     
                     let window = web_sys::window().unwrap();
                     if let Ok(resp_val) = JsFuture::from(window.fetch_with_request(&request)).await {
                         let resp: Response = resp_val.dyn_into().unwrap();
                         if resp.ok() {
                             if let Ok(json) = JsFuture::from(resp.json().unwrap()).await {
                                 if let Ok(data) = serde_wasm_bindgen::from_value::<UploadResponse>(json) {
                                     set_photo_link.set(data.url);
                                 }
                             }
                         }
                     }
                });
            }
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let upload_photo = move |_: Event| {};

    #[cfg(target_arch = "wasm32")]
    fn event_target_value(e: &Event) -> String {
        e.target().expect("target").dyn_into::<web_sys::HtmlInputElement>().expect("input element").value()
    }

    view! {
        <div style="max-width: 800px; margin: 0 auto;">
            <div style="display: flex; justify_content: space-between; align-items: center; margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">
                    "My Profile"
                </h1>
            </div>

            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; flex-direction: column; gap: 1.5rem;">
                <div style="display: flex; gap: 1rem;">
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"First Name"</label>
                        <input 
                            type="text" 
                            prop:value=first_name
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Last Name"</label>
                        <input 
                            type="text" 
                            prop:value=last_name
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                </div>

                <div style="display: flex; gap: 1rem;">
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Staff ID (Unique)"</label>
                        <input 
                            type="text" 
                            prop:value=staff_id_field
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Mobile Number"</label>
                        <input 
                            type="text" 
                            prop:value=mobile_number
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                </div>

                <div style="display: flex; gap: 1rem;">
                     <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"Username"</label>
                        <input 
                            type="text" 
                            prop:value=username
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                    <div style="display: flex; flex-direction: column; gap: 0.5rem; flex: 1;">
                        <label style="font-weight: 500;">"New Password"</label>
                        <input 
                            type="password" 
                            placeholder="Leave blank to keep current"
                            prop:value=password
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md);"
                        />
                    </div>
                </div>
                
                <div style="display: flex; flex-direction: column; gap: 0.5rem;">
                    <label style="font-weight: 500;">"Photo"</label>
                    <div style="display: flex; align-items: center; gap: 1rem;">
                        <img 
                             src=move || if photo_link.get().is_empty() { "https://ui-avatars.com/api/?name=User".to_string() } else { photo_link.get() }
                             alt="Preview"
                             style="width: 120px; height: 120px; border-radius: 50%; object-fit: cover; border: 4px solid var(--bg-surface); box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);"
                        />
                         <input 
                            type="file" 
                            accept="image/*"
                            disabled=true
                            style="padding: 0.75rem; border: 1px solid var(--border-input); border-radius: var(--radius-md); background-color: var(--bg-page); color: var(--text-muted);"
                        />
                    </div>
                </div>

                <div style="display: flex; justify-content: flex-end; gap: 1rem; margin-top: 1rem;">
                    <button 
                        on:click=save_profile
                        style="padding: 0.75rem 1.5rem; background-color: var(--brand-primary); color: white; border: none; border-radius: var(--radius-md); font-weight: 600; cursor: pointer;"
                    >
                        "Save Profile"
                    </button>
                </div>
            </div>
        </div>
    }
}
