//! Evidence search with keyword / tag / from / to filters wired to the
//! backend GET /evidence query parameters.

use leptos::*;
use crate::api::client;
use fieldtrace_shared::EvidenceResponse;

#[component]
pub fn EvidenceSearchPage() -> impl IntoView {
    let (keyword, set_keyword) = create_signal(String::new());
    let (tag, set_tag) = create_signal(String::new());
    let (from, set_from) = create_signal(String::new());
    let (to, set_to) = create_signal(String::new());
    let (results, set_results) = create_signal(Vec::<EvidenceResponse>::new());
    let (err, set_err) = create_signal(Option::<String>::None);

    let run_search = move || {
        let k = keyword.get(); let tg = tag.get();
        let fr = from.get(); let t2 = to.get();
        spawn_local(async move {
            match client::list_evidence(&k, &tg, &fr, &t2).await {
                Ok(list) => { set_results.set(list); set_err.set(None); }
                Err(e) => set_err.set(Some(e.message)),
            }
        });
    };

    // Load initial unfiltered results
    run_search.clone()();

    view! {
        <div class="card">
            <h2>"Evidence Search"</h2>
            {move || err.get().map(|e| view! { <div class="msg msg-error">{e}</div> })}
            <div class="filter-row">
                <input placeholder="keyword"
                    prop:value=keyword
                    on:input=move |e| set_keyword.set(event_target_value(&e)) />
                <input placeholder="tag"
                    prop:value=tag
                    on:input=move |e| set_tag.set(event_target_value(&e)) />
                <input type="date" placeholder="from"
                    prop:value=from
                    on:input=move |e| set_from.set(event_target_value(&e)) />
                <input type="date" placeholder="to"
                    prop:value=to
                    on:input=move |e| set_to.set(event_target_value(&e)) />
                <button class="btn" on:click=move |_| run_search()>"Search"</button>
            </div>
            <div class="list">
                {move || {
                    let items = results.get();
                    if items.is_empty() {
                        view! { <p class="muted">"No evidence matches those filters."</p> }.into_view()
                    } else {
                        items.into_iter().map(|e| view! {
                            <div class="list-item">
                                <strong>{e.filename.clone()}</strong>
                                <span class="tag tag-info">{e.media_type.clone()}</span>
                                {if e.missing_exif {
                                    view! { <span class="tag tag-error">"missing EXIF"</span> }.into_view()
                                } else { view! {}.into_view() }}
                                <span class="muted">" wm: "{e.watermark_text.clone()}</span>
                            </div>
                        }).collect_view()
                    }
                }}
            </div>
        </div>
    }
}
