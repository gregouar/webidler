use codee::binary::MsgpackSerdeCodec;
use leptos::html::*;
use leptos::prelude::*;
use leptos_use::{
    ReconnectLimit, UseWebSocketError, UseWebSocketOptions, UseWebSocketReturn,
    core::ConnectionReadyState, use_websocket_with_options,
};

use shared::client_messages::ClientConnectMessage;
use shared::client_messages::ClientMessage;
use shared::client_messages::TestMessage;
use shared::server_messages::ServerMessage;

use crate::components::ui::buttons::MainMenuButton;

const HEARTBEAT_PERIOD: u64 = 10_000;

#[component]
pub fn Connect() -> impl IntoView {
    let on_error_callback = move |e: UseWebSocketError<_, _>| {
        match e {
            UseWebSocketError::Event(e) => println!("[onerror]: event {:?}", e.type_()),
            _ => println!("[onerror]: {:?}", e),
        };
        // TODO: Toaster
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
            .immediate(false)
            .reconnect_limit(ReconnectLimit::Infinite)
            .on_error(on_error_callback), // .on_open(on_open_callback)
                                          // .on_close(on_close_callback)
                                          // .on_message(on_message_callback),
                                          // .heartbeat::<ClientMessage, MsgpackSerdeCodec>(HEARTBEAT_PERIOD),
    );

    let open_connection = move |_| {
        open();
    };

    {
        let send = send.clone();
        Effect::new(move |_| {
            if ready_state.get() == ConnectionReadyState::Open {
                let m = ClientMessage::Connect(ClientConnectMessage {
                    bearer: String::from("bearer token"),
                });
                send(&m);
            }
        });
    }

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
        ServerMessage::Update(m) => {
            println!("Got update: {:?}", m)
        }
    }
}
