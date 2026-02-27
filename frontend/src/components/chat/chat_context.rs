use leptos::prelude::*;
use shared::messages::chat::ChatMessage;

use crate::components::chat::{ring_buffer::RingBuffer, websocket::Websocket};

#[derive(Clone)]
pub struct ChatService {
    pub messages: RwSignal<RingBuffer<ChatMessage>>,
    pub send: Callback<String>,
}

#[component]
pub fn ChatProvider(url: String, children: Children) -> impl IntoView {
    let messages = RwSignal::new(RingBuffer::new(100));

    // // Create websocket here ONCE
    // Effect::new(move |_| {
    //     // connect websocket
    //     // register handlers
    //     // push messages into `messages`
    // });

    let send = Callback::new(move |msg: String| {
        // send through websocket
    });

    provide_context(ChatService { messages, send });

    view! { <Websocket url>{children()}</Websocket> }
}
