use std::sync::Arc;

use leptoaster::*;
use leptos::prelude::*;
use leptos::web_sys::CloseEvent;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use codee::binary::MsgpackSerdeCodec;

use shared::messages::client::ClientMessage;
use shared::messages::server::ServerMessage;

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
        (self.send)(message)
    }
}

#[component]
pub fn Websocket(children: Children) -> impl IntoView {
    let toaster = expect_toaster();
    let on_error_callback =
        move |e: UseWebSocketError<_, _>| toaster.error(format!("Connection error: {:?}", e));

    let toaster = expect_toaster();
    let on_close_callback = move |e: CloseEvent| {
        // TODO: if debug
        if !e.was_clean() {
            toaster.info(format!("Disconnected, trying to reconnect...",));
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
        "ws://127.0.0.1:4200/ws",
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

    view! {{children()}}
}
