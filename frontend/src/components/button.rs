use leptos::*;

#[component]
pub fn Button(
    children: Children,
    #[prop(optional, into)] on_click: Option<Callback<web_sys::MouseEvent>>,
    #[prop(optional, into)] class: String,
    #[prop(optional)] disabled: bool,
    #[prop(default = "submit")] type_: &'static str,
) -> impl IntoView {
    let base_style = "
        display: inline-flex;
        align-items: center;
        justify-content: center;
        padding: 0.75rem 1.5rem;
        font-weight: 600;
        border-radius: var(--radius-full);
        border: none;
        cursor: pointer;
        transition: all 0.2s ease;
        background-color: var(--brand-dark);
        color: var(--brand-primary);
        font-size: 1rem;
        text-transform: uppercase;
        letter-spacing: 0.02em;
    ";

    view! {
        <button
            type=type_
            class=format!("btn {}", class)
            style=base_style
            disabled=disabled
            on:click=move |ev| {
                if let Some(cb) = on_click.clone() {
                    cb.call(ev);
                }
            }
        >
            {children()}
        </button>
    }
}
