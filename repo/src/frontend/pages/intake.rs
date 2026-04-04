use leptos::*;
use crate::api::client;
use fieldtrace_shared::{IntakeRequest, IntakeResponse};

#[component]
pub fn IntakePage() -> impl IntoView {
    let (entries, set_entries) = create_signal(Vec::<IntakeResponse>::new());
    let (show_form, set_show_form) = create_signal(false);

    let refresh = move || {
        spawn_local(async move {
            if let Ok(list) = client::list_intake().await { set_entries.set(list); }
        });
    };
    refresh();

    view! {
        <div class="card">
            <h2>"Intake Records"</h2>
            <button class="btn" on:click=move |_| set_show_form.update(|v| *v = !*v)>
                {move || if show_form.get() { "Cancel" } else { "New Intake" }}
            </button>
            {move || show_form.get().then(|| {
                let refresh = refresh.clone();
                let set_show = set_show_form;
                view! { <IntakeForm on_done=move || { refresh(); set_show.set(false); } /> }
            })}
            <div class="list">
                {move || entries.get().into_iter().map(|r| {
                    let id = r.id.clone();
                    let status_class = match r.status.as_str() {
                        "received" => "tag-info",
                        "adopted" => "tag-ok",
                        _ => "tag-default",
                    };
                    view! {
                        <div class="list-item">
                            <strong>{r.intake_type.clone()}</strong>
                            <span class={format!("tag {}", status_class)}>{r.status.clone()}</span>
                            <span class="muted">" ID: "{id}</span>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn IntakeForm<F: Fn() + Clone + 'static>(on_done: F) -> impl IntoView {
    let (itype, set_itype) = create_signal("animal".to_string());
    let (details, set_details) = create_signal(String::new());
    let (err, set_err) = create_signal(Option::<String>::None);

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let req = IntakeRequest { intake_type: itype.get(), details: details.get() };
        let on_done = on_done.clone();
        spawn_local(async move {
            match client::create_intake(&req).await {
                Ok(_) => on_done(),
                Err(e) => set_err.set(Some(e.message)),
            }
        });
    };

    view! {
        <form class="addr-form" on:submit=submit>
            {move || err.get().map(|e| view! { <div class="msg msg-error">{e}</div> })}
            <select on:change=move |e| set_itype.set(event_target_value(&e))>
                <option value="animal">"Animal"</option>
                <option value="supply">"Supply"</option>
                <option value="donation">"Donation"</option>
            </select>
            <input placeholder="Details (JSON)" prop:value=details
                on:input=move |e| set_details.set(event_target_value(&e)) />
            <button type="submit" class="btn">"Create Intake"</button>
        </form>
    }
}
