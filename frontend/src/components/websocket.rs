use std::sync::Arc;

use leptos::prelude::*;
use leptos::web_sys::CloseEvent;
use leptos_use::{
    core::ConnectionReadyState, use_websocket_with_options, ReconnectLimit, UseWebSocketError,
    UseWebSocketOptions, UseWebSocketReturn,
};

use codee::binary::MsgpackSerdeCodec;

use shared::messages::client::ClientMessage;
use shared::messages::server::ServerMessage;

use crate::components::ui::toast::*;

const HEARTBEAT_PERIOD: u64 = 10_000;

#[derive(Clone)]
pub struct WebsocketContext {
    pub connected: Memo<bool>,
    pub message: Signal<Option<ServerMessage>>,
    // open: Arc<dyn Fn() + Send + Sync>,
    send: Arc<dyn Fn(&ClientMessage) + Send + Sync>,
    // close: Arc<dyn Fn() + Send + Sync>,
}

impl WebsocketContext {
    // create a method to avoid having to use parantheses around the field
    #[inline(always)]
    pub fn send(&self, message: &ClientMessage) {
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
                format!("Connection error: {:?}", e),
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
                    "Disconnected, trying to reconnect...",
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
    } = use_websocket_with_options::<ClientMessage, ServerMessage, MsgpackSerdeCodec, _, _>(
        &url,
        UseWebSocketOptions::default()
            // .immediate(false)
            .reconnect_limit(ReconnectLimit::Infinite)
            .on_error(on_error_callback)
            .on_close(on_close_callback)
            .heartbeat::<ClientMessage, MsgpackSerdeCodec>(HEARTBEAT_PERIOD),
    );

    let _ = open;
    let _ = close;
    provide_context(WebsocketContext {
        connected: Memo::new(move |_| ready_state.get() == ConnectionReadyState::Open),
        message,
        // open: Arc::new(open.clone()),
        send: Arc::new(send.clone()),
        // close: Arc::new(close.clone()),
    });

    view! { {children()} }
}
