use leptos::*;
use gloo_net::http::Request;

#[component]
pub fn ReportsPage() -> impl IntoView {
    let (summary, set_summary) = create_signal(Option::<serde_json::Value>::None);

    spawn_local(async move {
        if let Ok(resp) = Request::get("/reports/summary").send().await {
            if let Ok(v) = resp.json::<serde_json::Value>().await {
                set_summary.set(Some(v));
            }
        }
    });

    view! {
        <div class="card">
            <h2>"Dashboard Metrics"</h2>
            {move || summary.get().map(|v| view! {
                <div class="metrics">
                    <div class="metric"><strong>"Rescue Volume: "</strong>{v["rescue_volume"].to_string()}</div>
                    <div class="metric"><strong>"Donations Logged: "</strong>{v["donations_logged"].to_string()}</div>
                    <div class="metric"><strong>"Inventory on Hand: "</strong>{v["inventory_on_hand"].to_string()}</div>
                    <div class="metric"><strong>"Adoption Rate: "</strong>{v["adoption_conversion"].to_string()}</div>
                    <div class="metric"><strong>"Task Completion Rate: "</strong>{v["task_completion_rate"].to_string()}</div>
                </div>
            })}
        </div>
    }
}
