use leptos::*;
use crate::components::layout::DashboardLayout;
use shared::models::Product;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn DashboardPage() -> impl IntoView {
    #[allow(unused_variables)]
    let (products, set_products) = create_signal(Vec::<Product>::new());
    
    // Auto-fetch products on load (mocking the authenticated state for now, 
    // real app would need token persistence)
    create_effect(move |_| {
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            // NOTE: This will fail if not authenticated (token missing). 
            // Since we didn't persist token in login, this is a known limitation of this simple refactor.
            // Ideally we'd move fetching to a resource or context.
            // For now, let's just attempt fetch - if 401, user sees empty.
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            if let Ok(resp) = Request::get("/api/products")
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if let Ok(items) = resp.json::<Vec<Product>>().await {
                     set_products.set(items);
                 }
            }
        });
    });

    view! {
        <div style="display: flex; flex-direction: column; gap: 1rem;">
            <div style="padding-bottom: 2rem; border-bottom: 1px solid var(--border-subtle); margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Dashboard"</h1>
                <p style="color: var(--text-muted); margin-top: 0.5rem; font-size: 1.1rem;">"Welcome back to FastSales Overview"</p>
            </div>
            
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem;">
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem;">
                         <h3 style="font-size: 1rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">"Total Products"</h3>
                         <span style="font-size: 1.5rem;">"📦"</span>
                    </div>
                    <p style="font-size: 2.5rem; font-weight: 700; color: var(--brand-dark); line-height: 1;">
                        {move || products.get().len()}
                    </p>
                </div>
            </div>
             
             <div style="display: grid; gap: 1.5rem; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));">
                <For
                    each=move || products.get()
                    key=|product| product.id
                    children=move |product| view! {
                        <div style="padding: 1.5rem; background: var(--bg-surface); border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); transition: transform 0.2s, box-shadow 0.2s; cursor: pointer; display: flex; flex-direction: column; gap: 0.5rem;">
                            <div style="display: flex; justify-content: space-between; align-items: start;">
                                <h3 style="font-size: 1.25rem; font-weight: 600; margin: 0; color: var(--text-base);">{product.name}</h3>
                                <span style="background: var(--brand-light); color: var(--brand-primary); padding: 0.25rem 0.75rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; text-transform: uppercase;">{product.product_type.as_str()}</span>
                            </div>
                            <p style="color: var(--text-muted); font-size: 0.95rem; line-height: 1.5; flex: 1;">{product.description}</p>
                            <div style="margin-top: 1rem; padding-top: 1rem; border-top: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                                <span style="font-weight: 700; color: var(--brand-dark); font-size: 1.25rem;">
                                    {format!("${:.2}", product.price_cents as f64 / 100.0)}
                                </span>
                                <span style="font-size: 0.9rem; color: var(--text-muted);">
                                    {product.stock} " items"
                                </span>
                            </div>
                        </div>
                    }
                />
             </div>
        </div>
    }
}
