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
        width: 100%;
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
