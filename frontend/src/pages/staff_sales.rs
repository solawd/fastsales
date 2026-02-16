use leptos::*;
use leptos_router::*;
use shared::models::{Sale, Staff};
use crate::utils::CURRENCY;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
#[allow(unused_variables)]
pub fn StaffSalesPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.get().get("id").cloned().unwrap_or_default();

    let (staff_name, set_staff_name) = create_signal(String::new());
    let (sales, set_sales) = create_signal(Vec::<Sale>::new());
    
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    
    let (page, set_page) = create_signal(1);
    let limit = 20;

    let navigate = use_navigate();

    // Fetch staff details
    create_effect(move |_| {
        let staff_id = id();
        if !staff_id.is_empty() {
             #[cfg(target_arch = "wasm32")]
             spawn_local(async move {
                let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
                if let Ok(res) = Request::get(&format!("/api/staff/{}", staff_id))
                    .header("Authorization", &format!("Bearer {}", token))
                    .send().await {
                    if let Ok(data) = res.json::<Staff>().await {
                        set_staff_name.set(format!("{} {}", data.first_name, data.last_name));
                    }
                }
             });
        }
    });

    // Helper to extract value from event
    #[cfg(target_arch = "wasm32")]
    fn event_target_value(e: &web_sys::Event) -> String {
        use wasm_bindgen::JsCast;
        e.target().expect("target").dyn_into::<web_sys::HtmlInputElement>().expect("input element").value()
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn event_target_value(_: &web_sys::Event) -> String { String::new() }

    // Fetch sales function
    let fetch_sales = move || {
        let staff_id = id();
        let _navigate = navigate.clone();
        if staff_id.is_empty() { return; }
        
        // Capture values to avoid move issues
        let p = page.get();
        let s_date = start_date.get();
        let e_date = end_date.get();
        
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            let mut url = format!("/api/staff/{}/transactions", staff_id);
            let mut params = Vec::new();
            
            params.push(format!("page={}", p));
            params.push(format!("limit={}", limit));

            if !s_date.is_empty() {
                params.push(format!("start_date={}", s_date));
            }
            if !e_date.is_empty() {
                params.push(format!("end_date={}", e_date));
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

    // Initial fetch and refetch on page change
    create_effect({
        let fetch_sales = fetch_sales.clone();
        move |_| {
            // Depend on page to trigger refetch
            page.get(); 
            fetch_sales();
        }
    });

    view! {
        <div>
             <div style="margin-bottom: 2rem;">
                <A href="/staff" attr:style="color: var(--text-muted); text-decoration: none;">"← Back to Staff"</A>
                <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 0.5rem;">
                    <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">
                        {move || if staff_name.get().is_empty() { "Staff Sales".to_string() } else { format!("Sales by {}", staff_name.get()) }}
                    </h1>
                </div>
            </div>

            <div style="background: var(--bg-surface); padding: 1rem; border-radius: var(--radius-md); border: 1px solid var(--border-subtle); margin-bottom: 2rem;">
                <div style="display: flex; gap: 1rem; align-items: flex-end; flex-wrap: wrap;">
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
                            move |_| {
                                set_page.set(1); // Reset to page 1 on filter
                                fetch_sales();
                            }
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
                                        <td style="padding: 1rem;">
                                             <A href=format!("/sales/{}", sale.id) attr:style="text-decoration: none; color: var(--brand-primary); font-weight: 500;">"View"</A>
                                        </td>
                                    </tr>
                                }
                            }
                        />
                    </tbody>
                </table>
                 {move || if sales.get().is_empty() {
                    view! { <div style="padding: 2rem; text-align: center; color: var(--text-muted);">"No sales found."</div> }.into_view()
                } else {
                    view! { <div/> }.into_view()
                }}
            </div>
            
            <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 1rem;">
                <button
                    disabled=move || page.get() <= 1
                    on:click=move |_| set_page.update(|p| *p -= 1)
                    style="padding: 0.5rem 1rem; cursor: pointer;"
                    class:disabled=move || page.get() <= 1
                >
                    "Previous"
                </button>
                <span>{move || format!("Page {}", page.get())}</span>
                 <button
                    disabled=move || sales.get().len() < 20
                    on:click=move |_| set_page.update(|p| *p += 1)
                    style="padding: 0.5rem 1rem; cursor: pointer;"
                    class:disabled=move || sales.get().len() < 20
                >
                    "Next"
                </button>
            </div>
        </div>
    }
}
