use leptos::*;
use chrono::prelude::*;
use shared::models::{TopProduct, ProductSalesSummary};
use crate::utils::CURRENCY;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SalesReportsPage() -> impl IntoView {
    let now = Local::now();
    let today_str = now.format("%Y-%m-%d").to_string();
    let first_day_str = format!("{}-{:02}-01", now.year(), now.month());

    let (start_date, set_start_date) = create_signal(first_day_str.clone());
    let (end_date, set_end_date) = create_signal(today_str.clone());
    
    // Temporary state for inputs, init with defaults
    let (input_start_date, set_input_start_date) = create_signal(first_day_str);
    let (input_end_date, set_input_end_date) = create_signal(today_str);

    let (top_products, _set_top_products) = create_signal(Vec::<TopProduct>::new());
    let (product_sales, _set_product_sales) = create_signal(Vec::<ProductSalesSummary>::new());
    let (total_period_sales, set_total_period_sales) = create_signal(0i64);

    
    // Fetch Data Effect
    create_effect(move |_| {
        let _s_date = start_date.get();
        let _e_date = end_date.get();
        
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            // Build Query Params for Stats
            let mut stats_query = String::new();
            if !_s_date.is_empty() { stats_query.push_str(&format!("start_date={}&", _s_date)); }
            if !_e_date.is_empty() { stats_query.push_str(&format!("end_date={}", _e_date)); }
            
            // Fetch Sales By Product
            if let Ok(resp) = Request::get(&format!("/api/sales/stats/by_product?{}", stats_query))
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if let Ok(data) = resp.json::<Vec<ProductSalesSummary>>().await {
                     let total: i64 = data.iter().map(|d| d.total_amount_cents).sum();
                     set_total_period_sales.set(total);
                     _set_product_sales.set(data);
                 }
            }
            
            // Fetch Top Products
            if let Ok(resp) = Request::get(&format!("/api/sales_stats/top_products?{}", stats_query))
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if let Ok(data) = resp.json::<Vec<TopProduct>>().await {
                     _set_top_products.set(data);
                 }
            }
        });
    });

    let format_currency = |cents: i64| format!("{} {:.2}", CURRENCY, cents as f64 / 100.0);

    view! {
        <div style="display: flex; flex-direction: column; gap: 2rem;">
                <div style="padding-bottom: 1rem; border-bottom: 1px solid var(--border-subtle);">
                    <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Sales Reports"</h1>
                    <p style="color: var(--text-muted);">"Analyze sales performance over time"</p>
                </div>
                
                // Pane 1: Date Filters
                <div style="background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; align-items: flex-end; gap: 1rem;">
                    <div style="flex: 1;">
                        <label style="display: block; margin-bottom: 0.5rem; color: var(--text-muted); font-size: 0.9rem;">"Start Date"</label>
                        <input type="date"
                            on:input=move |ev| set_input_start_date.set(event_target_value(&ev))
                            prop:value=input_start_date
                            style="width: 100%; padding: 0.5rem; border: 1px solid var(--border-subtle); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div style="flex: 1;">
                        <label style="display: block; margin-bottom: 0.5rem; color: var(--text-muted); font-size: 0.9rem;">"End Date"</label>
                        <input type="date"
                            on:input=move |ev| set_input_end_date.set(event_target_value(&ev))
                            prop:value=input_end_date
                            style="width: 100%; padding: 0.5rem; border: 1px solid var(--border-subtle); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div>
                        <button 
                            on:click=move |_| {
                                set_start_date.set(input_start_date.get());
                                set_end_date.set(input_end_date.get());
                            }
                            style="padding: 0.5rem 1.5rem; background: var(--bg-page); color: var(--text-main); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); font-weight: 600; cursor: pointer;"
                        >
                            "Generate Report"
                        </button>
                    </div>
                </div>
                
                // Pane 2: Summary & Charts
                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 1.5rem; align-items: stretch;">
                   // Chart
                   <div style="background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                       <h3 style="margin-bottom: 1rem;">"Top Products"</h3>
                       <div style="height: 300px; position: relative; display: flex; flex-direction: column; align-items: center; justify-content: center;">
                           {move || {
                               let data = top_products.get();
                               if data.is_empty() {
                                   view! { <div style="color: var(--text-muted);">"No data available"</div> }.into_view()
                               } else {
                                   let total_all: i64 = data.iter().map(|d| d.total_sales_cents).sum();
                                   if total_all == 0 {
                                        return view! { <div style="color: var(--text-muted);">"No sales in period"</div> }.into_view();
                                   }

                                   let mut refined_data = Vec::new();
                                   for item in data.iter().take(3) {
                                       refined_data.push(item.clone());
                                   }
                                   
                                   let others_sum: i64 = data.iter().skip(3).map(|d| d.total_sales_cents).sum();
                                   if others_sum > 0 {
                                       refined_data.push(TopProduct { product_name: "Others".to_string(), total_sales_cents: others_sum });
                                   }
                                   
                                   let mut cumulative_percent = 0.0;
                                   let colors = vec!["#69BEEB", "#052850", "#f59e0b", "#9CA3AF"];
                                   
                                   view! {
                                       <svg viewBox="-1.2 -1.2 2.4 2.4" style="width: 100%; height: 100%; max-height: 250px; transform: rotate(-90deg);">
                                         {refined_data.into_iter().enumerate().map(|(i, item)| {
                                             let percent = item.total_sales_cents as f64 / total_all as f64;
                                             let start_angle = cumulative_percent * 2.0 * std::f64::consts::PI;
                                             cumulative_percent += percent;
                                             let end_angle = cumulative_percent * 2.0 * std::f64::consts::PI;
                                             
                                             let x1 = start_angle.cos();
                                             let y1 = start_angle.sin();
                                             let x2 = end_angle.cos();
                                             let y2 = end_angle.sin();
                                             
                                             let large_arc_flag = if percent > 0.5 { 1 } else { 0 };
                                             
                                             if percent > 0.999 {
                                                 view! { <circle cx="0" cy="0" r="1" fill=colors[i % colors.len()] /> }.into_view()
                                             } else {
                                                 view! {
                                                     <path d=format!("M 0 0 L {} {} A 1 1 0 {} 1 {} {} Z", x1, y1, large_arc_flag, x2, y2) fill=colors[i % colors.len()] stroke="white" stroke-width="0.02" />
                                                 }.into_view()
                                             }
                                         }).collect::<Vec<_>>()}
                                       </svg>
                                   }.into_view()
                               }
                           }}
                       </div>
                       
                       // Legend
                       <div style="margin-top: 1rem; display: flex; flex-wrap: wrap; gap: 1rem; font-size: 0.8rem; justify-content: center;">
                            {move || {
                               let data = top_products.get();
                               let total_all: i64 = data.iter().map(|d| d.total_sales_cents).sum();
                               if total_all == 0 { return view! {}.into_view(); }

                               let mut refined_data = Vec::new();
                               for item in data.iter().take(3) { refined_data.push(item.clone()); }
                               let others_sum: i64 = data.iter().skip(3).map(|d| d.total_sales_cents).sum();
                               if others_sum > 0 { refined_data.push(TopProduct { product_name: "Others".to_string(), total_sales_cents: others_sum }); }
                               
                               let colors = vec!["#69BEEB", "#052850", "#f59e0b", "#9CA3AF"];
                               refined_data.into_iter().enumerate().map(|(i, item)| {
                                   let percent = (item.total_sales_cents as f64 / total_all as f64) * 100.0;
                                   view! {
                                       <div style="display: flex; align-items: center; gap: 0.5rem;">
                                           <span style=format!("width: 12px; height: 12px; background: {}; border-radius: 2px;", colors[i % colors.len()])></span>
                                           <span>{item.product_name} " (" {format!("{:.1}%", percent)} ")"</span>
                                       </div>
                                   }
                               }).collect::<Vec<_>>().into_view()
                            }}
                       </div>
                   </div>

                   // Total Sales Huge Display
                   <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); display: flex; flex-direction: column; justify-content: center; align-items: center;">
                        <h3 style="color: var(--text-muted); text-transform: uppercase;">"Total Period Sales"</h3>
                        <div style="font-size: 3.5rem; font-weight: 800; color: var(--brand-dark); margin: 1rem 0;">
                            {move || format_currency(total_period_sales.get())}
                        </div>
                   </div>
                </div>
                
                // Pane 3: Product Sales Table
                <div style="background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                    <h3 style="margin-bottom: 1rem;">"Sales Details"</h3>
                    <div style="overflow-x: auto;">
                        <table style="width: 100%; border-collapse: collapse; font-size: 0.9rem;">
                            <thead>
                                <tr style="border-bottom: 2px solid var(--border-subtle); text-align: left;">
                                    <th style="padding: 0.75rem;">"Product"</th>
                                    <th style="padding: 0.75rem;">"Quantity Purchased"</th>
                                    <th style="padding: 0.75rem;">"Total Amount Generated"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    product_sales.get().into_iter().map(|sale| {
                                        view! {
                                            <tr style="border-bottom: 1px solid var(--border-subtle);">
                                                <td style="padding: 0.75rem;">{sale.product_name}</td>
                                                <td style="padding: 0.75rem;">{sale.total_quantity}</td>
                                                <td style="padding: 0.75rem;">{format_currency(sale.total_amount_cents)}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>
    }
}
