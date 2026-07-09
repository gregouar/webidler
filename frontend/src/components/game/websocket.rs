use std::sync::Arc;
use std::{
    cell::{Cell, RefCell},
    thread_local,
};

use leptos::prelude::*;
use leptos::web_sys::CloseEvent;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use codee::{Decoder, Encoder, binary::MsgpackSerdeCodec};
use serde::{Serialize, de::DeserializeOwned};

use shared::messages::client::ClientMessage;
use shared::messages::compression;
use shared::messages::server::ServerMessage;

use crate::components::ui::toast::*;

const HEARTBEAT_PERIOD: u64 = 10_000;

thread_local! {
    static COMPRESSION_DICTIONARY: RefCell<Option<Vec<u8>>> = const { RefCell::new(None) };
    static COMPRESSION_ENABLED: Cell<bool> = const { Cell::new(false) };
}

fn set_compression_dictionary(dictionary: Vec<u8>) {
    COMPRESSION_DICTIONARY.with(|stored| {
        *stored.borrow_mut() = (!dictionary.is_empty()).then_some(dictionary);
    });
    COMPRESSION_ENABLED.with(|enabled| enabled.set(true));
}

fn clear_compression_dictionary() {
    COMPRESSION_DICTIONARY.with(|stored| {
        stored.borrow_mut().take();
    });
    COMPRESSION_ENABLED.with(|enabled| enabled.set(false));
}

fn encode_with_current_dictionary(raw: Vec<u8>) -> Result<Vec<u8>, String> {
    if !COMPRESSION_ENABLED.with(Cell::get) {
        return Ok(raw);
    }

    COMPRESSION_DICTIONARY.with(|stored| {
        let stored = stored.borrow();
        compression::encode_payload_with_dictionary(raw, stored.as_deref())
            .map_err(|e| e.to_string())
    })
}

fn decode_with_current_dictionary(val: &[u8]) -> Result<std::borrow::Cow<'_, [u8]>, String> {
    COMPRESSION_DICTIONARY.with(|stored| {
        let stored = stored.borrow();
        compression::decode_payload_with_dictionary(val, stored.as_deref())
            .map_err(|e| e.to_string())
    })
}

struct CompressedMsgpackSerdeCodec;

impl<T> Encoder<T> for CompressedMsgpackSerdeCodec
where
    T: Serialize,
{
    type Error = String;
    type Encoded = Vec<u8>;

    fn encode(val: &T) -> Result<Self::Encoded, Self::Error> {
        let encoded = MsgpackSerdeCodec::encode(val).map_err(|e| e.to_string())?;
        encode_with_current_dictionary(encoded)
    }
}

impl<T> Decoder<T> for CompressedMsgpackSerdeCodec
where
    T: DeserializeOwned,
{
    type Error = String;
    type Encoded = [u8];

    fn decode(val: &Self::Encoded) -> Result<T, Self::Error> {
        let decoded = decode_with_current_dictionary(val)?;
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
    let handshake_complete = RwSignal::new(false);

    Effect::new(move |_| {
        if ready_state.get() != ConnectionReadyState::Open {
            clear_compression_dictionary();
            handshake_complete.set(false);
        }
    });

    Effect::new(move |_| {
        if let Some(ServerMessage::Connect(connect_message)) = message.get() {
            set_compression_dictionary(connect_message.compression_dictionary);
            handshake_complete.set(true);
        }
    });

    provide_context(WebsocketContext {
        connected: Memo::new(move |_| {
            ready_state.get() == ConnectionReadyState::Open && handshake_complete.get()
        }),
        message,
        // open: Arc::new(open.clone()),
        send: Arc::new(send.clone()),
        // close: Arc::new(close.clone()),
    });

    view! { {children()} }
}
