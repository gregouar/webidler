use codee::{Encoder, binary::MsgpackSerdeCodec};
use leptos::prelude::*;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};
use shared::data::item::ItemSpecs;
use std::{
    collections::{HashMap, HashSet},
    ops::ControlFlow,
    sync::Arc,
};
use web_sys::CloseEvent;

use shared_chat::{
    messages::{
        client::{ClientChatMessage, ClientConnectMessage, ClientPostMessage},
        server::{ErrorType, ServerChatMessage},
    },
    ring_buffer::RingBuffer,
    types::{ChatChannel, ChatContent, ChatMessage, LinkedItemBytes, UserId},
};

use crate::components::{accessibility::AccessibilityContext, auth::AuthContext, ui::toast::*};

const HEARTBEAT_PERIOD: u64 = 10_000;

#[derive(Clone)]
pub struct ChatContext {
    pub user_id: RwSignal<Option<UserId>>,

    pub users_map: RwSignal<HashMap<UserId, String>>,
    // TODO: Split in multiple buckets to keep longer system message than global
    pub messages: RwSignal<RingBuffer<ChatMessage>>,
    pub send: Callback<String>,

    pub minimized: RwSignal<bool>,
    pub opened: RwSignal<bool>,
    pub selected_channels: RwSignal<HashSet<ChatChannel>>,
    pub write_channel: RwSignal<ChatChannel>,
    pub linked_item: RwSignal<Option<Arc<ItemSpecs>>>,
}

impl ChatContext {
    pub fn link_item(&self, item_specs: Arc<ItemSpecs>) {
        self.linked_item.set(Some(item_specs.clone()));
        self.opened.set(true);
        self.minimized.set(false);
    }
}

#[component]
pub fn ChatProvider(url: String, children: Children) -> impl IntoView {
    let accessibility_context: AccessibilityContext = expect_context();
    let opened = RwSignal::new(!accessibility_context.is_on_mobile());

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
                && opened.get()
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
    let write_channel = RwSignal::new(ChatChannel::Global);
    let linked_item = RwSignal::new(None);

    let send = Callback::new(move |msg| {
        if let Ok(content) = ChatContent::try_new(msg) {
            send(
                &ClientPostMessage {
                    channel: write_channel.get_untracked(),
                    content,
                    linked_item: linked_item.read_untracked().as_ref().and_then(
                        |linked_item: &Arc<ItemSpecs>| {
                            // let mut item_specs = (**linked_item).clone();
                            // item_specs.signature = Default::default();

                            let serialized_item = LinkedItemBytes::try_new(
                                MsgpackSerdeCodec::encode(linked_item).ok()?,
                            )
                            .ok()?;

                            // Some((serialized_item, linked_item.signature.clone()))
                            Some(serialized_item)
                        },
                    ),
                }
                .into(),
            );
            linked_item.set(None);
        }
    });

    let chat_context = ChatContext {
        user_id: RwSignal::new(None),
        send,
        users_map: Default::default(),
        messages: RwSignal::new(RingBuffer::new(100)),
        // TODO: Store in storage
        minimized: RwSignal::new(true),
        opened,
        selected_channels: RwSignal::new(HashSet::from([
            ChatChannel::Global,
            ChatChannel::Trade,
            ChatChannel::System,
        ])),
        write_channel,
        linked_item,
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
            chat_context.user_id.set(Some(m.user_id));
            for message in m.history.into_iter().rev() {
                push_message(chat_context, message)
            }
        }
        ServerChatMessage::Error(error_message) => {
            let toaster: Toasts = expect_context();
            show_toast(
                toaster,
                format!("Chat: {}", error_message.message),
                match error_message.error_type {
                    ErrorType::Server => ToastVariant::Error,
                    ErrorType::Chat => ToastVariant::Warning,
                },
            );
            if error_message.must_disconnect {
                return ControlFlow::Break(());
            }
        }
        ServerChatMessage::Broadcast(m) => push_message(chat_context, *m),
        ServerChatMessage::WhisperFeedback(m) => {
            // TODO: Local users map
            if let Some(username) = m.target_username
                && !chat_context
                    .users_map
                    .read_untracked()
                    .contains_key(&m.target_user_id)
            {
                chat_context
                    .users_map
                    .write()
                    .insert(m.target_user_id, username);
            }
            chat_context.messages.write().push(m.chat_message);
            chat_context
                .write_channel
                .set(ChatChannel::Whisper(m.target_user_id));
        }
    }
    ControlFlow::Continue(())
}

fn push_message(chat_context: &ChatContext, message: ChatMessage) {
    if let Some((user_id, username)) = message.user_id.zip(message.username.clone())
        && !chat_context
            .users_map
            .read_untracked()
            .contains_key(&user_id)
    {
        chat_context.users_map.write().insert(user_id, username);
    }

    if !chat_context
        .messages
        .read_untracked()
        .iter()
        .any(|m| m.sent_at == message.sent_at && m.user_id == message.user_id)
    {
        chat_context.messages.write().push(message);
    }
}
