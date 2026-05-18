use std::sync::Arc;

use leptos::leptos_dom::logging::console_log;
use leptos::prelude::*;
use leptos::web_sys::CloseEvent;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use codee::{Decoder, Encoder, binary::MsgpackSerdeCodec};
use serde::{Serialize, de::DeserializeOwned};

use shared::messages::client::ClientMessage;
use shared::messages::server::ServerMessage;

use crate::components::ui::toast::*;

const HEARTBEAT_PERIOD: u64 = 10_000;

struct CompressedMsgpackSerdeCodec;

impl<T> Encoder<T> for CompressedMsgpackSerdeCodec
where
    T: Serialize,
{
    type Error = String;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        let encoded = MsgpackSerdeCodec::encode(val).map_err(|e| e.to_string())?;
        shared::messages::compression::encode_payload(encoded).map_err(|e| e.to_string())
    }
}

impl<T> Decoder<T> for CompressedMsgpackSerdeCodec
where
    T: DeserializeOwned,
{
    type Error = String;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        let decoded =
            shared::messages::compression::decode_payload(val).map_err(|e| e.to_string())?;
        console_log(&format!(
            "{:.0}%: {} vs {}",
            (val.len() as f64 / decoded.len() as f64) * 100.0,
            val.len(),
            decoded.len()
        ));
        MsgpackSerdeCodec::decode(&decoded).map_err(|e| e.to_string())
    }
}

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
                format!("Game connection error: {e:?}"),
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
                    "Game disconnected, trying to reconnect...",
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
    } = use_websocket_with_options::<ClientMessage, ServerMessage, CompressedMsgpackSerdeCodec, _, _>(
        &url,
        UseWebSocketOptions::default()
            // .immediate(false)
            .reconnect_limit(ReconnectLimit::Infinite)
            .on_error(on_error_callback)
            .on_close(on_close_callback)
            .heartbeat::<ClientMessage, CompressedMsgpackSerdeCodec>(HEARTBEAT_PERIOD),
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
