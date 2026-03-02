use codee::binary::MsgpackSerdeCodec;
use leptos::prelude::*;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};
use std::ops::ControlFlow;
use web_sys::CloseEvent;

use shared_chat::{
    messages::{
        client::{ClientChatMessage, ClientConnectMessage, ClientPostMessage},
        server::{ErrorType, ServerChatMessage},
    },
    ring_buffer::RingBuffer,
    types::{ChatChannel, ChatContent, ChatMessage},
};

use crate::components::{auth::AuthContext, ui::toast::*};

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
    Effect::new({
        let close = close.clone();
        move || {
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
        }
    });

    Effect::new({
        let send = send.clone();
        move || {
            if auth.is_authenticated() && ready_state.get() == ConnectionReadyState::Open {
                send(&ClientConnectMessage { jwt: auth.token() }.into())
            }
        }
    });

    // Chat
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

    let chat_context = ChatContext {
        messages: RwSignal::new(RingBuffer::new(100)),
        send,
    };

    Effect::new({
        let chat_context = chat_context.clone();
        move |_| {
            if let Some(message) = message.get() {
                match handle_message(&chat_context, message) {
                    ControlFlow::Continue(_) => {}
                    ControlFlow::Break(_) => close(),
                }
            }
        }
    });

    provide_context(chat_context);

    view! { {children()} }
}

fn handle_message(chat_context: &ChatContext, message: ServerChatMessage) -> ControlFlow<()> {
    match message {
        ServerChatMessage::Connect(m) => {
            chat_context.messages.write().extend(m.history.into_iter());
        }
        // ServerChatMessage::Disconnect(_) => {
        //     return ControlFlow::Break(());
        // }
        // chat_context.messages.write().push(ChatMessage {
        //     channel: ChatChannel::System,
        //     user_id: None,
        //     user_name: None,
        //     content: "disconnected".into(),
        //     sent_at: Utc::now(),
        // }),
        ServerChatMessage::Error(error_message) => {
            let toaster: Toasts = expect_context();
            show_toast(
                toaster,
                error_message.message,
                match error_message.error_type {
                    ErrorType::Server => ToastVariant::Error,
                    ErrorType::Chat => ToastVariant::Warning,
                },
            );
            if error_message.must_disconnect {
                return ControlFlow::Break(());
            }
        }
        ServerChatMessage::Broadcast(m) => chat_context.messages.write().push(*m),
    }
    return ControlFlow::Continue(());
}
