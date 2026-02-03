use leptos::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    let layout_style = "
        min-height: 100vh;
        display: flex;
        flex-direction: column;
        align-items: center;
        background-color: var(--bg-page);
    ";
    
    let header_style = "
        width: 100%;
        padding: 1.5rem 2rem;
        display: flex;
        align-items: center;
        gap: 1rem;
        background-color: var(--bg-surface);
        border-bottom: 1px solid var(--border-subtle);
    ";

    let logo_style = "height: 40px; width: auto;";
    let brand_style = "font-family: var(--font-heading); font-weight: 700; font-size: 1.25rem; color: var(--brand-dark);";

    let content_style = "
        width: 100%;
        max-width: 1200px;
        padding: 2rem;
        flex: 1;
    ";

    view! {
        <div style=layout_style>
            <header style=header_style>
                <img src="/fs_logo.png" alt="FastSales Logo" style=logo_style />
                <span style=brand_style>"FastSales"</span>
            </header>
            <main style=content_style>
                {children()}
            </main>
        </div>
    }
}

#[component]
pub fn DashboardLayout() -> impl IntoView {
    use crate::components::sidebar::Sidebar;
    use leptos_router::Outlet;

    let layout_style = "
        min-height: 100vh;
        display: flex;
        background-color: var(--bg-page);
    ";
    
    let main_style = "
        display: flex;
        flex-direction: column;
        flex: 1;
        width: 100%;
    ";

    let content_style = "
        width: 100%;
        padding: 2rem;
        flex: 1;
    ";

    view! {
        <div style=layout_style>
            <Sidebar />
            <div style=main_style>
                <main style=content_style>
                    <Outlet/>
                </main>
            </div>
        </div>
    }
}
