use leptos::*;
use fieldtrace_shared::UserResponse;

#[component]
pub fn Nav<F: Fn() + Clone + 'static>(
    user: ReadSignal<Option<UserResponse>>,
    on_logout: F,
) -> impl IntoView {
    view! {
        <header class="app-header">
            <h1>"FieldTrace"</h1>
            <div class="nav-spacer"></div>
            {move || user.get().map(|u| {
                let role_label = match u.role.as_str() {
                    "administrator" => "Admin",
                    "operations_staff" => "Staff",
                    "auditor" => "Auditor",
                    _ => "User",
                }.to_string();
                let username = u.username.clone();
                let on_logout = on_logout.clone();
                view! {
                    <span class="nav-user">{username}" ("{role_label}")"</span>
                    <button class="nav-logout" on:click=move |_| on_logout()>"Logout"</button>
                }
            })}
        </header>
    }
}
