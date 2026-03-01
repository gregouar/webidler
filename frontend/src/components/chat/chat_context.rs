use codee::binary::MsgpackSerdeCodec;
use leptos::prelude::*;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};
use shared::{
    messages::{
        chat::{ChatChannel, ChatMessage, ClientChatMessage, ClientPostMessage, ServerChatMessage},
        server::ErrorType,
    },
    types::ChatContent,
};
use web_sys::CloseEvent;

use crate::components::{auth::AuthContext, chat::ring_buffer::RingBuffer, ui::toast::*};

const HEARTBEAT_PERIOD: u64 = 10_000;
#[derive(Clone)]
pub struct ChatContext {
    pub messages: RwSignal<RingBuffer<ChatMessage>>,
    pub send: Callback<String>,
}

#[component]
pub fn ChatProvider(url: String, children: Children) -> impl IntoView {
    // Websocket
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

    // Chat
    let messages = RwSignal::new(RingBuffer::new(100));

    let send = Callback::new(move |msg: String| {
        if let Ok(content) = ChatContent::try_new(msg) {
            send(
                &ClientPostMessage {
                    channel: ChatChannel::Global,
                    content,
                }
                .into(),
            );
        }
    });

    let chat_context = ChatContext { messages, send };

    Effect::new({
        let chat_context = chat_context.clone();
        move |_| {
            if let Some(message) = message.get() {
                handle_message(&chat_context, message);
            }
        }
    });

    provide_context(chat_context);

    view! { {children()} }
}

fn handle_message(chat_context: &ChatContext, message: ServerChatMessage) {
    match message {
        ServerChatMessage::Connect(_) => {}
        ServerChatMessage::Disconnect(_) => {}
        ServerChatMessage::Error(error_message) => {
            let toaster: Toasts = expect_context();
            show_toast(
                toaster,
                error_message.message,
                match error_message.error_type {
                    ErrorType::Server => ToastVariant::Error,
                    ErrorType::Game => ToastVariant::Warning,
                    ErrorType::Chat => ToastVariant::Warning,
                },
            );
            if error_message.must_disconnect {
                let navigate = leptos_router::hooks::use_navigate();
                navigate("/", Default::default());
            }
        }
        ServerChatMessage::Broadcast(m) => chat_context.messages.write().push(m.chat_message),
    }
}
