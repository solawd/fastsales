use leptos::*;
use leptos_router::use_navigate;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;
#[cfg(target_arch = "wasm32")]
use serde::{Deserialize, Serialize};

use crate::components::button::Button;
use crate::components::input::Input;

#[cfg(target_arch = "wasm32")]
#[derive(Serialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[cfg(target_arch = "wasm32")]
#[derive(Deserialize)]
struct AuthResponse {
    token: String,
    token_type: String,
    expires_in: u64,
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    #[allow(unused_variables)]
    let _navigate = use_navigate();

    let on_submit = move |_| {
        let username = username.get_untracked();
        let password = password.get_untracked();
        let set_error = set_error.clone();
        let _navigate = _navigate.clone();
        
        #[cfg(target_arch = "wasm32")]
        {
            spawn_local(async move {
                set_error.set(None);
                let payload = LoginPayload { username, password };
                let response = match Request::post("/api/auth/login")
                    .header("Content-Type", "application/json")
                    .json(&payload)
                {
                    Ok(req) => req.send().await,
                    Err(_) => {
                        set_error.set(Some("Network error".to_string()));
                        return;
                    }
                };

                let response = match response {
                    Ok(resp) => resp,
                    Err(_) => {
                        set_error.set(Some("Request failed".to_string()));
                        return;
                    }
                };

                if !response.ok() {
                    set_error.set(Some("Invalid credentials".to_string()));
                    return;
                }

                if let Ok(auth_data) = response.json::<AuthResponse>().await {
                     if let Ok(Some(storage)) = web_sys::window().unwrap().local_storage() {
                         let _ = storage.set_item("jwt_token", &auth_data.token);
                     }
                }
                
                _navigate("/dashboard", Default::default());
            });
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (username, password);
             set_error.set(Some("Browser required.".to_string()));
        }
    };

    let centered_box = "
        max_width: 320px;
        width: 60%;
        margin: 4rem auto;
        padding: 2rem;
        background: var(--bg-surface);
        border-radius: var(--radius-lg);
        box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    ";

    let page_style = "
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        align-items: center;
        background-color: var(--bg-page);
        padding-top: 2rem;
    ";

    view! {
        <div style=page_style>
            <div style=centered_box>
                <div style="text-align: center; margin-bottom: 2rem;">
                    <img src="/fs_logo.png" alt="FastSales Logo" style="height: 80px; width: auto; margin-bottom: 1rem;" />
                    <h1 style="font-family: var(--font-heading); color: var(--brand-dark); font-size: 2.5rem; margin: 0;">"FastSales"</h1>
                </div>
                <h2 style="text-align: center; margin-bottom: 0.5rem; color: var(--text-muted); font-size: 1.2rem;">"Staff Login"</h2>
                <Input
                    label="Username"
                    type_="text"
                    placeholder="Enter your username"
                    value=username
                    set_value=set_username
                />
                <Input
                    label="Password"
                    type_="password"
                    placeholder="Enter your password"
                    value=password
                    set_value=set_password
                />
                
                <Show when=move || error.get().is_some()>
                    <p style="color: #ef4444; font-size: 0.9rem; margin: 0;">{move || error.get().unwrap_or_default()}</p>
                </Show>

                <Button on_click=on_submit>"Sign In"</Button>
            </div>
        </div>
    }
}
