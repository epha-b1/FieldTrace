use leptos::*;

use crate::api::client;
use crate::app::Page;
use crate::pages::address_book::AddressBookPage;
use crate::pages::evidence_search::EvidenceSearchPage;
use crate::pages::intake::IntakePage;
use crate::pages::reports::ReportsPage;
use crate::pages::workspace::WorkspacePage;
use fieldtrace_shared::UserResponse;

#[component]
pub fn DashboardPage(
    user: ReadSignal<Option<UserResponse>>,
    set_page: WriteSignal<Page>,
    set_user: WriteSignal<Option<UserResponse>>,
) -> impl IntoView {
    let (health, set_health) = create_signal(Option::<Result<String, String>>::None);
    let (delete_msg, set_delete_msg) = create_signal(Option::<String>::None);

    spawn_local(async move {
        let result = client::check_health().await;
        set_health.set(Some(result));
    });

    // Session-expiry check: periodically verify session by calling /auth/me
    {
        let set_page = set_page;
        let set_user = set_user;
        spawn_local(async move {
            gloo_timers::future::sleep(std::time::Duration::from_secs(60)).await;
            if client::get_me().await.is_err() {
                set_user.set(None);
                set_page.set(Page::Login);
            }
        });
    }

    let request_delete = move |_| {
        spawn_local(async move {
            match client::request_account_deletion().await {
                Ok(v) => set_delete_msg.set(Some(
                    v.get("message").and_then(|m| m.as_str()).unwrap_or("Deletion requested").to_string(),
                )),
                Err(e) => set_delete_msg.set(Some(e.message)),
            }
        });
    };

    let cancel_delete = move |_| {
        spawn_local(async move {
            match client::cancel_account_deletion().await {
                Ok(_) => set_delete_msg.set(Some("Deletion cancelled".into())),
                Err(e) => set_delete_msg.set(Some(e.message)),
            }
        });
    };

    view! {
        <div class="app-body">
            <div class="card">
                <h2>"System Status"</h2>
                {move || match health.get() {
                    None => view! {
                        <span class="status-indicator status-loading">
                            <span class="dot dot-loading"></span>
                            "Checking..."
                        </span>
                    }.into_view(),
                    Some(Ok(s)) => view! {
                        <span class="status-indicator status-ok">
                            <span class="dot dot-ok"></span>
                            {format!("System: {}", s)}
                        </span>
                    }.into_view(),
                    Some(Err(e)) => view! {
                        <span class="status-indicator status-error">
                            <span class="dot dot-error"></span>
                            {format!("Error: {}", e)}
                        </span>
                    }.into_view(),
                }}
            </div>

            <div class="card">
                <h2>"Profile"</h2>
                {move || user.get().map(|u| view! {
                    <div class="profile-info">
                        <p><strong>"Username: "</strong>{u.username.clone()}</p>
                        <p><strong>"Role: "</strong>{u.role.clone()}</p>
                    </div>
                })}
                <div class="account-lifecycle">
                    <h3>"Account Lifecycle"</h3>
                    <p class="muted">"Requesting deletion starts a 7-day cooling-off window. You can cancel anytime during that period."</p>
                    {move || delete_msg.get().map(|m| view! { <div class="msg msg-info">{m}</div> })}
                    <button class="btn btn-danger" on:click=request_delete>"Request Account Deletion"</button>
                    <button class="btn" on:click=cancel_delete>"Cancel Deletion"</button>
                </div>
            </div>

            <WorkspacePage />
            <ReportsPage />
            <IntakePage />
            <EvidenceSearchPage />
            <AddressBookPage />
        </div>
    }
}
