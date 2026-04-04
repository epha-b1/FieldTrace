use leptos::*;

use crate::api::client;
use crate::app::Page;
use crate::pages::address_book::AddressBookPage;
use crate::pages::intake::IntakePage;
use fieldtrace_shared::UserResponse;

#[component]
pub fn DashboardPage(
    user: ReadSignal<Option<UserResponse>>,
    set_page: WriteSignal<Page>,
    set_user: WriteSignal<Option<UserResponse>>,
) -> impl IntoView {
    let (health, set_health) = create_signal(Option::<Result<String, String>>::None);

    spawn_local(async move {
        let result = client::check_health().await;
        set_health.set(Some(result));
    });

    // Session-expiry check: periodically verify session by calling /auth/me
    {
        let set_page = set_page;
        let set_user = set_user;
        spawn_local(async move {
            // Wait a bit, then check session validity
            gloo_timers::future::sleep(std::time::Duration::from_secs(60)).await;
            if client::get_me().await.is_err() {
                set_user.set(None);
                set_page.set(Page::Login);
            }
        });
    }

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
            </div>

            <IntakePage />
            <AddressBookPage />

            <div class="card">
                <h2>"Workspace"</h2>
                <p style="color: var(--color-muted);">
                    "Intake queue, pending inspections, and exceptions will appear here."
                </p>
            </div>
        </div>
    }
}
