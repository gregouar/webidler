use codee::string::FromToStringCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket, use_websocket_with_options,
};

use crate::components::ui::buttons::MainMenuButton;

#[component]
pub fn Connect() -> impl IntoView {
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        open,
        close,
        ..
    } = use_websocket::<String, String, FromToStringCodec>("ws://127.0.0.1:4200/ws");
    // } = use_websocket::<JsonValue, JsonValue, MsgpackSerdeCodec>("wss://127.0.0.8:4200/ws");

    let send_message = move |_| {
        let m = "hello".to_string();
        send(&m);
    };

    let status = move || ready_state.get().to_string();

    let connected = move || ready_state.get() == ConnectionReadyState::Open;

    let open_connection = move |_| {
        open();
    };
    let close_connection = move |_| {
        close();
    };

    // Effect::new(move |_| {
    //     message.with(move |message| {
    //         if let Some(m) = message {
    //             update_history(&set_history, format!("[message]: {:?}", m));
    //         }
    //     })
    // });

    view! {
        <main class="my-0 mx-auto text-center text-white font-serif">
            <p>"status: " {status}</p>

            <MainMenuButton on:click=open_connection prop:disabled=connected>
                "Open"
            </MainMenuButton>
            <MainMenuButton on:click=close_connection prop:disabled=move || !connected()>
                "Close"
            </MainMenuButton>
            <MainMenuButton on:click=send_message prop:disabled=move || !connected()>
                "Send"
            </MainMenuButton>

            <p>"Receive message: " {move || format!("{:?}", message.get())}</p>
        </main>
    }
}
