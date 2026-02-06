use leptos::*;
use leptos::*;
use shared::models::{TopProduct, SalesListResponse, SalesStats}; // Ensure these are imported
use crate::utils::CURRENCY;
#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

#[component]
pub fn SalesReportsPage() -> impl IntoView {
    let (start_date, set_start_date) = create_signal(String::new());
    let (end_date, set_end_date) = create_signal(String::new());
    let (top_products, set_top_products) = create_signal(Vec::<TopProduct>::new());
    let (sales_response, set_sales_response) = create_signal(SalesListResponse {
        sales: vec![],
        total_sales_period_cents: 0,
    });
    let (page, set_page) = create_signal(1i64);
    let (limit, set_limit) = create_signal(20i64); // Default 20
    
    // Fetch Data Effect
    create_effect(move |_| {
        let s_date = start_date.get();
        let e_date = end_date.get();
        let p = page.get();
        let l = limit.get();
        
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            // Build Query Params
            let mut query = format!("page={}&limit={}", p, l);
            if !s_date.is_empty() { query.push_str(&format!("&start_date={}", s_date)); }
            if !e_date.is_empty() { query.push_str(&format!("&end_date={}", e_date)); }
            
            // Fetch Sales List
            if let Ok(resp) = Request::get(&format!("/api/sales?{}", query))
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if let Ok(data) = resp.json::<SalesListResponse>().await {
                     set_sales_response.set(data);
                 }
            }
            
            // Fetch Top Products (only depends on date, not page)
            let mut stats_query = String::new();
            if !s_date.is_empty() { stats_query.push_str(&format!("start_date={}&", s_date)); }
            if !e_date.is_empty() { stats_query.push_str(&format!("end_date={}", e_date)); }
            
            if let Ok(resp) = Request::get(&format!("/api/sales_stats/top_products?{}", stats_query))
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if let Ok(data) = resp.json::<Vec<TopProduct>>().await {
                     set_top_products.set(data);
                 }
            }
        });
    });

    let format_currency = |cents: i64| format!("{}{:.2}", CURRENCY, cents as f64 / 100.0);

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
                            on:input=move |ev| set_start_date.set(event_target_value(&ev))
                            prop:value=start_date
                            style="width: 100%; padding: 0.5rem; border: 1px solid var(--border-subtle); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div style="flex: 1;">
                        <label style="display: block; margin-bottom: 0.5rem; color: var(--text-muted); font-size: 0.9rem;">"End Date"</label>
                        <input type="date"
                            on:input=move |ev| set_end_date.set(event_target_value(&ev))
                            prop:value=end_date
                            style="width: 100%; padding: 0.5rem; border: 1px solid var(--border-subtle); border-radius: var(--radius-md);"
                        />
                    </div>
                    <div>
                        <button 
                            on:click=move |_| {
                                set_start_date.set(String::new());
                                set_end_date.set(String::new());
                                set_page.set(1);
                            }
                            style="padding: 0.5rem 1.5rem; background: var(--bg-page); color: var(--text-main); border: 1px solid var(--border-subtle); border-radius: var(--radius-md); font-weight: 600; cursor: pointer;"
                        >
                            "Reset"
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
                            {move || format_currency(sales_response.get().total_sales_period_cents)}
                        </div>
                   </div>
                </div>
                
                // Pane 3: Paginated Table
                <div style="background: var(--bg-surface); padding: 1.5rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle);">
                    <h3 style="margin-bottom: 1rem;">"Sales Details"</h3>
                    <div style="overflow-x: auto;">
                        <table style="width: 100%; border-collapse: collapse; font-size: 0.9rem;">
                            <thead>
                                <tr style="border-bottom: 2px solid var(--border-subtle); text-align: left;">
                                    <th style="padding: 0.75rem;">"Date"</th>
                                    <th style="padding: 0.75rem;">"Qty"</th>
                                    <th style="padding: 0.75rem;">"Total"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || {
                                    sales_response.get().sales.into_iter().map(|sale| {
                                        view! {
                                            <tr style="border-bottom: 1px solid var(--border-subtle);">
                                                <td style="padding: 0.75rem;">{sale.date_of_sale.format("%Y-%m-%d %H:%M").to_string()}</td>
                                                <td style="padding: 0.75rem;">{sale.quantity}</td>
                                                <td style="padding: 0.75rem;">{format_currency(sale.total_resolved)}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()
                                }}
                            </tbody>
                        </table>
                    </div>
                    
                    // Pagination
                    <div style="display: flex; justify-content: flex-end; gap: 1rem; margin-top: 1rem; align-items: center;">
                        <button 
                            on:click=move |_| set_page.update(|p| *p = (*p - 1).max(1))
                            disabled=move || page.get() <= 1
                            style="padding: 0.5rem 1rem; border: 1px solid var(--border-subtle); background: white; border-radius: var(--radius-md); cursor: pointer;"
                        >
                            "Previous"
                        </button>
                        <span>"Page " {move || page.get()}</span>
                         <button 
                            on:click=move |_| set_page.update(|p| *p = *p + 1)
                            disabled=move || sales_response.get().sales.len() < limit.get() as usize
                            style="padding: 0.5rem 1rem; border: 1px solid var(--border-subtle); background: white; border-radius: var(--radius-md); cursor: pointer;"
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
    }
}
