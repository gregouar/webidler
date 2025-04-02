use codee::binary::MsgpackSerdeCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos_use::{
    UseWebSocketOptions, UseWebSocketReturn, core::ConnectionReadyState, use_websocket_with_options,
};

use shared::client_messages::ClientConnectMessage;
use shared::client_messages::ClientMessage;
use shared::client_messages::TestMessage;
use shared::server_messages::ServerMessage;

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
    } = use_websocket_with_options::<ClientMessage, ServerMessage, MsgpackSerdeCodec, _, _>(
        "ws://127.0.0.1:4200/ws",
        UseWebSocketOptions::default(),
        // Enable heartbeats every 10 seconds. In this case we use the same codec as for the
        // other messages. But this is not necessary.
        // .heartbeat::<ClientMessage, MsgpackSerdeCodec>(10_000),
    );

    let open_connection = move |_| {
        open();
    };

    let send2 = send.clone();
    Effect::new(move |_| {
        if ready_state.get() == ConnectionReadyState::Open {
            let m = ClientMessage::Connect(ClientConnectMessage {
                greeting: String::from("pouhello"),
                value: 42,
            });
            send2(&m);
        }
    });

    let send_message = move |_| {
        let m = ClientMessage::Test(TestMessage {
            greeting: String::from("test"),
            value: 3,
        });
        send(&m);
    };

    let status = move || ready_state.get().to_string();

    let connected = move || ready_state.get() == ConnectionReadyState::Open;

    let close_connection = move |_| {
        close();
    };
    Effect::new(move |_| {
        message.with(move |message| {
            if let Some(message) = message {
                process_message(message);
            }
        })
    });

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

fn process_message(message: &ServerMessage) {
    match message {
        ServerMessage::Connect(m) => {
            println!("Got hello: {:?}", m)
        }
    }
}
