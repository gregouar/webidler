use anyhow::Result;

use axum::{
    body::Bytes,
    extract::{
        connect_info::ConnectInfo,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use axum_extra::TypedHeader;

use std::net::SocketAddr;
use std::ops::ControlFlow;

use futures::{sink::SinkExt, stream::StreamExt};

use shared::{
    client_messages::ClientMessage,
    server_messages::{ServerConnectMessage, ServerMessage},
};

// TODO: Nice wrapper around ServerMessage to have into traits?
fn into_ws_msg(message: &ServerMessage) -> Result<Message> {
    Ok(Message::Binary(Bytes::from_owner(rmp_serde::to_vec(
        message,
    )?)))
}

fn from_ws_msg(message: &Bytes) -> Result<ClientMessage> {
    Ok(rmp_serde::from_slice(message)?)
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");

    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        let n_msg = 20;
        for _ in 0..n_msg {
            if let Ok(m) = into_ws_msg(&ServerMessage::Connect(ServerConnectMessage {
                greeting: String::from("imma server"),
                value: 69,
            })) {
                if sender.send(m).await.is_err() {
                    return;
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        }

        if let Err(e) = sender
            .send(Message::Close(Some(CloseFrame {
                code: axum::extract::ws::close_code::NORMAL,
                reason: Utf8Bytes::from_static("Goodbye"),
            })))
            .await
        {
            println!("Could not send Close due to {e}, probably it is ok?");
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if process_message(msg, who).is_break() {
                break;
            }
        }
    });

    tokio::select! {
        r = (&mut send_task) => {
            if let Err(e) = r {
                println!("Error sending messages {e:?}")

            }
            recv_task.abort();
        },
        r = (&mut recv_task) => {
            if let Err(e) = r {
                println!("Error receiving messages {e:?}");
            }
            send_task.abort();
        }
    }

    // returning from the handler closes the websocket connection
    println!("Websocket context {who} destroyed");
}

fn process_message(msg: Message, who: SocketAddr) -> ControlFlow<(), ()> {
    match msg {
        Message::Binary(b) => match from_ws_msg(&b) {
            Ok(m) => handle_client_message(m),
            Err(_) => {
                println!(">>> {who} sent invalid message, closing");
                return ControlFlow::Break(());
            }
        },
        Message::Text(_) => {
            println!(">>> {who} sent str instead of bytes, closing");
            return ControlFlow::Break(());
        }
        Message::Close(c) => {
            if let Some(cf) = c {
                println!(
                    ">>> {} sent close with code {} and reason `{}`",
                    who, cf.code, cf.reason
                );
            } else {
                println!(">>> {who} somehow sent close message without CloseFrame");
            }
            return ControlFlow::Break(());
        }
        _ => {} // Ignore ping pong
    }
    ControlFlow::Continue(())
}

fn handle_client_message(msg: ClientMessage) {
    match msg {
        ClientMessage::Heartbeat => {}
        ClientMessage::Connect(m) => {
            println!("Connect: {:?}", m)
        }
        ClientMessage::Test(m) => {
            println!("Test: {:?}", m)
        }
    }
}
