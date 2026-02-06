use leptos::*;
use leptos_router::{A, use_location, use_navigate};

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    let _navigate = use_navigate();
    
    let handle_logout = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let _ = window().local_storage().unwrap().unwrap().remove_item("jwt_token");
            _navigate("/", Default::default());
        }
    };
    let sidebar_style = "
        width: 250px;
        background-color: var(--bg-surface);
        border-right: 1px solid var(--border-subtle);
        height: 100vh;
        position: sticky;
        top: 0;
        display: flex;
        flex-direction: column;
        padding: 2rem 1rem;
    ";

    let nav_style = "
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    ";


    
    // Helper to style active links could be added here later with use_location

    let logo_style = "width: 100%; height: auto; margin-bottom: 0.5rem; border-radius: var(--radius-md);";
    let ul_style = "list-style-type: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.5rem;";

    view! {
        <aside style=sidebar_style>
            <div style="margin-bottom: 2rem;">
                <img src="/fs_logo.png" alt="FastSales Logo" style=logo_style />
            </div>
            <nav style=nav_style>
                <ul style=ul_style>
                    <li>
                        <A href="/dashboard" class={move || if location.pathname.get() == "/dashboard" || location.pathname.get() == "/" { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
                            "Dashboard"
                        </A>
                    </li>
                    <li>
                        <A href="/products" class={move || if location.pathname.get().starts_with("/products") { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
                            "Products"
                        </A>
                    </li>
                    <li>
                        <A href="/sales" class={move || if location.pathname.get().starts_with("/sales") { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="23 6 13.5 15.5 8.5 10.5 1 18"></polyline><polyline points="17 6 23 6 23 12"></polyline></svg>
                            "Sales"
                        </A>
                    </li>
                    <li>
                        <A href="/reports" class={move || if location.pathname.get().starts_with("/reports") { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.21 15.89A10 10 0 1 1 8 2.83"></path><path d="M22 12A10 10 0 0 0 12 2v10z"></path></svg>
                            "Reports"
                        </A>
                    </li>
                    <li>
                        <A href="/customers" class={move || if location.pathname.get().starts_with("/customers") { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
                            "Customers"
                        </A>
                    </li>
                    <li>
                        <A href="/staff" class={move || if location.pathname.get().starts_with("/staff") { "sidebar-link active" } else { "sidebar-link" }}>
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="8.5" cy="7" r="4"></circle><polyline points="17 11 19 13 23 9"></polyline></svg>
                            "Staff"
                        </A>
                    </li>
                </ul>
            </nav>
            <div style="margin-top: auto;">
                <A href="/profile" class={move || if location.pathname.get() == "/profile" { "sidebar-link active" } else { "sidebar-link" }} attr:style="margin-bottom: 0.5rem;">
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"></path><circle cx="12" cy="7" r="4"></circle></svg>
                    "My Profile"
                </A>
                <button 
                    on:click=handle_logout
                    class="logout-btn"
                >
                    <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"></path><polyline points="16 17 21 12 16 7"></polyline><line x1="21" y1="12" x2="9" y2="12"></line></svg>
                    "Sign Out"
                </button>
            </div>
        </aside>
    }
}
