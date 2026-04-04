use leptos::*;

use crate::api::client;
use crate::app::Page;
use fieldtrace_shared::UserResponse;

#[component]
pub fn RegisterPage(
    set_page: WriteSignal<Page>,
    set_user: WriteSignal<Option<UserResponse>>,
) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        let u = username.get();
        let p = password.get();

        if p.len() < 12 {
            set_error.set(Some("Password must be at least 12 characters".into()));
            set_loading.set(false);
            return;
        }

        spawn_local(async move {
            match client::register(&u, &p).await {
                Ok(auth) => {
                    set_user.set(Some(auth.user));
                    set_page.set(Page::Dashboard);
                }
                Err(e) => {
                    set_error.set(Some(e.message));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <div class="auth-container">
            <div class="auth-card">
                <h1 class="auth-title">"FieldTrace"</h1>
                <h2>"Create Administrator Account"</h2>
                <p class="auth-subtitle">
                    "This creates the first administrator. Registration is disabled after setup."
                </p>

                {move || error.get().map(|e| view! {
                    <div class="msg msg-error">{e}</div>
                })}

                <form on:submit=on_submit>
                    <label>"Username"</label>
                    <input
                        type="text"
                        prop:value=username
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                        required=true
                    />
                    <label>"Password"<small>" (min 12 characters)"</small></label>
                    <input
                        type="password"
                        prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required=true
                        minlength=12
                    />
                    <button type="submit" disabled=loading>
                        {move || if loading.get() { "Creating..." } else { "Create Account" }}
                    </button>
                </form>

                <p class="auth-link">
                    "Already set up? "
                    <a href="#" on:click=move |ev| {
                        ev.prevent_default();
                        set_page.set(Page::Login);
                    }>"Sign In"</a>
                </p>
            </div>
        </div>
    }
}
