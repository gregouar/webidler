use std::sync::Arc;

use leptoaster::*;
use leptos::prelude::*;
use leptos::web_sys::CloseEvent;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use codee::binary::MsgpackSerdeCodec;

use shared::messages::client::ClientConnectMessage;
use shared::messages::client::ClientMessage;
use shared::messages::client::TestMessage;
use shared::messages::server::ServerMessage;

const HEARTBEAT_PERIOD: u64 = 10_000;

#[derive(Clone)]
pub struct WebsocketContext {
    pub connected: Signal<bool>,
    pub message: Signal<Option<ServerMessage>>,
    send: Arc<dyn Fn(&ClientMessage) + Send + Sync>, // use Arc to make it easily cloneable
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

    // let toaster = expect_toaster();
    // let on_message_callback = move |message: &ServerMessage| {
    //     if let Some(toast) = process_message(message) {
    //         toaster.info(toast);
    //     }
    // };

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
            // .on_open(on_open_callback)
            .on_close(on_close_callback)
            // .on_message(on_message_callback)
            .heartbeat::<ClientMessage, MsgpackSerdeCodec>(HEARTBEAT_PERIOD),
    );

    // let open_connection = move |_| {
    //     open();
    // };

    // {
    //     let send = send.clone();
    //     Effect::new(move |_| {
    //         if ready_state.get() == ConnectionReadyState::Open {
    //             send(
    //                 &ClientConnectMessage {
    //                     bearer: String::from("Le Pou"),
    //                 }
    //                 .into(),
    //             );
    //         }
    //     });
    // }

    // let send_message = move |_| {
    //     send(
    //         &TestMessage {
    //             greeting: String::from("test"),
    //             value: 3,
    //         }
    //         .into(),
    //     );
    // };

    // let status = move || ready_state.get().to_string();

    // Effect::new(move |_| set_connected.set(ready_state.get() == ConnectionReadyState::Open));

    // let close_connection = move |_| {
    //     close();
    // };

    let connected = RwSignal::new(false);
    Effect::new(move |_| connected.set(ready_state.get() == ConnectionReadyState::Open));

    provide_context(WebsocketContext {
        connected: connected.into(),
        message,
        send: Arc::new(send.clone()),
    });

    view! {{children()}}
}
