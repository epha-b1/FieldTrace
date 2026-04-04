use leptos::*;

use crate::api::client;
use crate::components::nav::Nav;
use crate::pages::{dashboard::DashboardPage, login::LoginPage, register::RegisterPage};
use fieldtrace_shared::UserResponse;

#[derive(Clone, Debug, PartialEq)]
pub enum Page {
    Loading,
    Login,
    Register,
    Dashboard,
}

#[component]
pub fn App() -> impl IntoView {
    let (page, set_page) = create_signal(Page::Loading);
    let (user, set_user) = create_signal(Option::<UserResponse>::None);
    let (session_msg, set_session_msg) = create_signal(Option::<String>::None);

    // On mount: check current auth state
    {
        let set_page = set_page;
        let set_user = set_user;
        spawn_local(async move {
            match client::get_me().await {
                Ok(u) => {
                    set_user.set(Some(u));
                    set_page.set(Page::Dashboard);
                }
                Err(_) => {
                    set_page.set(Page::Login);
                }
            }
        });
    }

    let do_logout = move || {
        let set_page = set_page;
        let set_user = set_user;
        let set_session_msg = set_session_msg;
        spawn_local(async move {
            let _ = client::logout().await;
            set_user.set(None);
            set_session_msg.set(Some("You have been logged out.".into()));
            set_page.set(Page::Login);
        });
    };

    view! {
        {move || {
            let p = page.get();
            match p {
                Page::Loading => view! {
                    <div class="center-box">
                        <span class="status-indicator status-loading">
                            <span class="dot dot-loading"></span>
                            "Loading..."
                        </span>
                    </div>
                }.into_view(),

                Page::Login => view! {
                    <LoginPage
                        set_page=set_page
                        set_user=set_user
                        session_msg=session_msg
                        set_session_msg=set_session_msg
                    />
                }.into_view(),

                Page::Register => view! {
                    <RegisterPage set_page=set_page set_user=set_user />
                }.into_view(),

                Page::Dashboard => view! {
                    <Nav user=user on_logout=do_logout.clone() />
                    <DashboardPage user=user set_page=set_page set_user=set_user />
                }.into_view(),
            }
        }}
    }
}
