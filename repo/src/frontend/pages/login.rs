use leptos::*;

use crate::api::client;
use crate::app::Page;
use fieldtrace_shared::UserResponse;

#[component]
pub fn LoginPage(
    set_page: WriteSignal<Page>,
    set_user: WriteSignal<Option<UserResponse>>,
    session_msg: ReadSignal<Option<String>>,
    set_session_msg: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (error, set_error) = create_signal(Option::<String>::None);
    let (loading, set_loading) = create_signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);
        set_session_msg.set(None);

        let u = username.get();
        let p = password.get();
        spawn_local(async move {
            match client::login(&u, &p).await {
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
                <h2>"Sign In"</h2>

                {move || session_msg.get().map(|m| view! {
                    <div class="msg msg-info">{m}</div>
                })}

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
                    <label>"Password"</label>
                    <input
                        type="password"
                        prop:value=password
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        required=true
                    />
                    <button type="submit" disabled=loading>
                        {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>

                <p class="auth-link">
                    "First time setup? "
                    <a href="#" on:click=move |ev| {
                        ev.prevent_default();
                        set_page.set(Page::Register);
                    }>"Create Administrator"</a>
                </p>
            </div>
        </div>
    }
}
