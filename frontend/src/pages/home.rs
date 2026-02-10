use leptos::*;

use shared::models::SalesStats;

#[cfg(target_arch = "wasm32")]
use gloo_net::http::Request;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::spawn_local;

use crate::utils::CURRENCY;

#[component]
pub fn DashboardPage() -> impl IntoView {
    let (today_sales, _set_today_sales) = create_signal(SalesStats::default());
    let (weekly_sales, _set_weekly_sales) = create_signal(Vec::<shared::models::DailySales>::new());
    
    let _navigate = leptos_router::use_navigate();

    create_effect(move |_| {
        let _navigate = _navigate.clone();
        #[cfg(target_arch = "wasm32")]
        spawn_local(async move {
            let token = web_sys::window().unwrap().local_storage().unwrap().unwrap().get_item("jwt_token").unwrap().unwrap_or_default();
            
            // Fetch Today's Stats
            if let Ok(resp) = Request::get("/api/sales/stats/today")
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if resp.status() == 401 {
                     _navigate("/", Default::default());
                     return;
                 }
                 if let Ok(stats) = resp.json::<SalesStats>().await {
                     _set_today_sales.set(stats);
                 }
            }

            // Fetch Weekly Stats
            if let Ok(resp) = Request::get("/api/sales/stats/week")
                .header("Authorization", &format!("Bearer {}", token))
                .send().await {
                 if resp.status() == 401 {
                     _navigate("/", Default::default());
                     return;
                 }
                 if let Ok(stats) = resp.json::<Vec<shared::models::DailySales>>().await {
                     _set_weekly_sales.set(stats);
                 }
            }
        });
    });

    // Helper to format currency
    let format_currency = |cents: i64| format!("{} {:.2}", CURRENCY, cents as f64 / 100.0);

    view! {
        <div style="display: flex; flex-direction: column; gap: 1rem;">
            <div style="padding-bottom: 2rem; border-bottom: 1px solid var(--border-subtle); margin-bottom: 2rem;">
                <h1 style="font-size: 2rem; font-weight: 700; color: var(--text-heading);">"Dashboard"</h1>
                <p style="color: var(--text-muted); margin-top: 0.5rem; font-size: 1.1rem;">"Welcome back to FastSales Overview"</p>
            </div>
            
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 1.5rem;">
                // Total Sales Today Card
                <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
                    <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 1rem;">
                         <h3 style="font-size: 1rem; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.05em;">"Total Sales Today"</h3>
                         <span style="font-size: 1.5rem;">"💰"</span>
                    </div>
                    <p style="font-size: 2.5rem; font-weight: 700; color: var(--brand-dark); line-height: 1;">
                        {move || format_currency(today_sales.get().total_sales_cents)}
                    </p>
                </div>
                
                 // Total Sales Count Today Card (Bonus?) - reusing the space since products count is gone?
                 // Let's just keep today's sales for now as requested.
            </div>
             
            // Chart Section
            <div style="background: var(--bg-surface); padding: 2rem; border-radius: var(--radius-lg); border: 1px solid var(--border-subtle); margin-top: 1rem;">
                <h3 style="font-size: 1.25rem; font-weight: 600; color: var(--text-heading); margin-bottom: 2rem;">"Weekly Sales Trend"</h3>
                
                <div style="width: 100%; height: 400px; position: relative;">
                    {move || {
                        let data = weekly_sales.get();
                        let days_labels = vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
                        
                        // Always calculate max based on available data, default to 100 if empty
                        let max_sales = data.iter().map(|d| d.total_sales_cents).max().unwrap_or(100) as f64;
                        let max_sales = if max_sales == 0.0 { 100.0 } else { max_sales };

                        let max_count = data.iter().map(|d| d.count).max().unwrap_or(10) as f64;
                        let max_count = if max_count == 0.0 { 10.0 } else { max_count };

                        // Map data to coordinates
                        // X axis: 5 to 95 (7 data points) -> 5% margin on each side
                        let get_x = |i: usize| 5.0 + (i as f64 / 6.0) * 90.0;
                        let get_y_sales = |val: i64| 100.0 - ((val as f64 / max_sales) * 80.0);
                        let get_y_count = |val: i64| 100.0 - ((val as f64 / max_count) * 80.0);

                        let sales_points: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, d)| {
                            (get_x(i), get_y_sales(d.total_sales_cents))
                        }).collect();

                        let count_points: Vec<(f64, f64)> = data.iter().enumerate().map(|(i, d)| {
                            (get_x(i), get_y_count(d.count))
                        }).collect();

                        // Helper to generate smoothed path
                        let generate_smooth_path = |points: &Vec<(f64, f64)>| -> String {
                            if points.len() < 2 {
                                String::new()
                            } else {
                                let mut d = format!("M {:.2},{:.2}", points[0].0, points[0].1);
                                for i in 0..points.len()-1 {
                                    let p0 = if i > 0 { points[i-1] } else { points[i] };
                                    let p1 = points[i];
                                    let p2 = points[i+1];
                                    let p3 = if i + 2 < points.len() { points[i+2] } else { p2 };

                                    let cp1x = p1.0 + (p2.0 - p0.0) / 6.0;
                                    let cp1y = p1.1 + (p2.1 - p0.1) / 6.0;

                                    let cp2x = p2.0 - (p3.0 - p1.0) / 6.0;
                                    let cp2y = p2.1 - (p3.1 - p1.1) / 6.0;

                                    d.push_str(&format!(" C {:.2},{:.2} {:.2},{:.2} {:.2},{:.2}", cp1x, cp1y, cp2x, cp2y, p2.0, p2.1));
                                }
                                d
                            }
                        };

                        let sales_path_d = generate_smooth_path(&sales_points);
                        let count_path_d = generate_smooth_path(&count_points);
                        
                        // Close path for fill (Sales only, to keep it clean)
                        let sales_fill_d = if !sales_points.is_empty() {
                            format!("{} L {:.2},100 L {:.2},100 Z", sales_path_d, sales_points.last().unwrap().0, sales_points.first().unwrap().0)
                        } else {
                            String::new()
                        };

                        view! {
                            <div style="display: flex; gap: 1rem; margin-bottom: 1rem; font-size: 0.9rem;">
                                <div style="display: flex; align-items: center; gap: 0.5rem; color: var(--text-muted);">
                                    <span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: var(--brand-primary);"></span>
                                    {format!("Total Sales ({})", CURRENCY)}
                                </div>
                                <div style="display: flex; align-items: center; gap: 0.5rem; color: var(--text-muted);">
                                    <span style="display: inline-block; width: 10px; height: 10px; border-radius: 50%; background: #f59e0b;"></span>
                                    "Sales Count (#)"
                                </div>
                            </div>

                            <div style="position: relative; width: 100%; height: 100%;">
                                <svg width="100%" height="100%" viewBox="0 0 100 100" preserveAspectRatio="none" style="overflow: visible; position: absolute; top: 0; left: 0;">
                                    // Gradient Definition
                                    <defs>
                                        <linearGradient id="chartGradient" x1="0" x2="0" y1="0" y2="1">
                                            <stop offset="0%" stop-color="var(--brand-primary)" stop-opacity="0.1"/>
                                            <stop offset="100%" stop-color="var(--brand-primary)" stop-opacity="0"/>
                                        </linearGradient>
                                    </defs>
                                    
                                    // Grid lines
                                    {
                                        (0..=4).map(|i| {
                                            let y = 20.0 + (i as f64 * 20.0);
                                            view! {
                                                <line x1="0" y1=y x2="100" y2=y stroke="var(--border-subtle)" stroke-width="0.5" stroke-dasharray="2" />
                                            }
                                        }).collect::<Vec<_>>()
                                    }
                                    
                                    // Sales Area Fill
                                    <path d=sales_fill_d fill="url(#chartGradient)" />

                                    // Sales Line (Primary)
                                    <path 
                                        d=sales_path_d 
                                        fill="none" 
                                        stroke="var(--brand-primary)" 
                                        stroke-width="2" 
                                        vector-effect="non-scaling-stroke"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    />

                                    // Count Line (Secondary - Orange/Amber)
                                    <path 
                                        d=count_path_d 
                                        fill="none" 
                                        stroke="#f59e0b" 
                                        stroke-width="2" 
                                        vector-effect="non-scaling-stroke"
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                    />
                                    
                                    // Points (Circles) - Still inside SVG but they might be ovals. 
                                    // If we want perfect circles we should move them to HTML or use non-scaling markers (masked).
                                    // For now, let's keep them here as requested to only fix textual components.
                                    {sales_points.iter().map(|(x, y)| {
                                         view! {
                                            <circle cx=*x cy=*y r="1.5" fill="white" stroke="var(--brand-primary)" stroke-width="0.5" vector-effect="non-scaling-stroke" />
                                         }
                                    }).collect::<Vec<_>>()}

                                    {count_points.iter().map(|(x, y)| {
                                         view! {
                                            <circle cx=*x cy=*y r="1.5" fill="white" stroke="#f59e0b" stroke-width="0.5" vector-effect="non-scaling-stroke" />
                                         }
                                    }).collect::<Vec<_>>()}
                                </svg>
                                
                                // HTML Text Overlays
                                // Sales Values
                                {sales_points.into_iter().enumerate().map(|(i, (x, y))| {
                                     let val = data[i].total_sales_cents;
                                     view! {
                                        <div style=format!("position: absolute; left: {}%; top: {}%; transform: translate(-50%, -100%); margin-top: -8px; font-size: 0.75rem; font-weight: 600; color: var(--brand-dark); pointer-events: none;", x, y)>
                                            {format!("{} {}", CURRENCY, val / 100)}
                                        </div>
                                     }
                                }).collect::<Vec<_>>()}

                                // Count Values
                                {count_points.into_iter().enumerate().map(|(i, (x, y))| {
                                     let val = data[i].count;
                                     view! {
                                        <div style=format!("position: absolute; left: {}%; top: {}%; transform: translate(-50%, -100%); margin-top: -12px; font-size: 0.75rem; font-weight: 600; color: #d97706; pointer-events: none;", x, y)>
                                            {format!("#{}", val)}
                                        </div>
                                     }
                                }).collect::<Vec<_>>()}
                                
                                // X-Axis Labels
                                {days_labels.into_iter().enumerate().map(|(i, label)| {
                                    let x = 5.0 + (i as f64 / 6.0) * 90.0;
                                    view! {
                                        <div style=format!("position: absolute; left: {}%; bottom: -25px; transform: translateX(-50%); font-size: 0.8rem; color: var(--text-muted);", x)>
                                            {label}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
