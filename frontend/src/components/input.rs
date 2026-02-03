use leptos::*;
use leptos::event_target_value;

#[component]
pub fn Input(
    #[prop(into)] label: String,
    #[prop(into)] type_: String,
    #[prop(into)] placeholder: String,
    #[prop(into)] value: Signal<String>,
    #[prop(into)] set_value: WriteSignal<String>,
) -> impl IntoView {
    let container_style = "display: flex; flex-direction: column; gap: 0.5rem; width: 100%;";
    let label_style = "font-weight: 500; font-size: 0.9rem; color: var(--text-muted);";
    let input_style = "
        padding: 0.8rem 1rem;
        border-radius: var(--radius-md);
        border: 1px solid var(--border-subtle);
        font-size: 1rem;
        width: 100%;
        background: var(--bg-surface);
        transition: border-color 0.2s;
    ";

    view! {
        <label style=container_style>
            <span style=label_style>{label}</span>
            <input
                type=type_
                placeholder=placeholder
                prop:value=move || value.get()
                on:input=move |ev| set_value.set(event_target_value(&ev))
                style=input_style
            />
        </label>
    }
}
