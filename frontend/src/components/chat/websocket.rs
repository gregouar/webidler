use std::sync::Arc;

use leptos::{prelude::*, web_sys::CloseEvent};
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use codee::binary::MsgpackSerdeCodec;

use shared::messages::chat::{ClientChatMessage, ServerChatMessage};

use crate::components::{auth::AuthContext, ui::toast::*};

const HEARTBEAT_PERIOD: u64 = 10_000;

#[derive(Clone)]
pub struct WebsocketContext {
    pub connected: Memo<bool>,
    pub message: Signal<Option<ServerChatMessage>>,
    send: Arc<dyn Fn(&ClientChatMessage) + Send + Sync>,
}

impl WebsocketContext {
    // create a method to avoid having to use parantheses around the field
    #[inline(always)]
    pub fn send(&self, message: &ClientChatMessage) {
        // TODO: Add constraint/limit rates?
        (self.send)(message)
    }
}

#[component]
pub fn Websocket(url: String, children: Children) -> impl IntoView {
    let on_error_callback = {
        let toaster = expect_context::<Toasts>();
        move |e: UseWebSocketError<_, _>| {
            show_toast(
                toaster,
                format!("Chat connection error: {e:?}"),
                ToastVariant::Error,
            )
        }
    };

    let on_close_callback = {
        let toaster = expect_context::<Toasts>();
        move |e: CloseEvent| {
            // TODO: if debug
            if !e.was_clean() {
                show_toast(
                    toaster,
                    "Chat disconnected, trying to reconnect...",
                    ToastVariant::Info,
                )
            }
        }
    };

    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        open,
        close,
        ..
    } = use_websocket_with_options::<ClientChatMessage, ServerChatMessage, MsgpackSerdeCodec, _, _>(
        &url,
        UseWebSocketOptions::default()
            .immediate(false)
            .reconnect_limit(ReconnectLimit::Infinite)
            .on_error(on_error_callback)
            .on_close(on_close_callback)
            .heartbeat::<ClientChatMessage, MsgpackSerdeCodec>(HEARTBEAT_PERIOD),
    );

    let auth: AuthContext = expect_context();
    Effect::new(move || {
        let is_auth = auth.is_authenticated();
        let state = ready_state.get_untracked();

        if is_auth
            && state != ConnectionReadyState::Open
            && state != ConnectionReadyState::Connecting
        {
            open();
        }

        if !is_auth && state == ConnectionReadyState::Open {
            close();
        }
    });

    provide_context(WebsocketContext {
        connected: Memo::new(move |_| ready_state.get() == ConnectionReadyState::Open),
        message,
        // open: Arc::new(open.clone()),
        send: Arc::new(send.clone()),
        // close: Arc::new(close.clone()),
    });

    view! { {children()} }
}
