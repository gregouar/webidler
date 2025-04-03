use anyhow::Result;

use std::{
    ops::ControlFlow,
    time::{Duration, Instant},
};

use shared::{
    client_messages::ClientMessage,
    server_messages::{ConnectMessage, ServerMessage, UpdateMessage},
};

use crate::websocket::WebSocketConnection;

const LOOP_MIN_PERIOD: Duration = Duration::from_millis(100);
pub struct GameInfo {}

pub async fn run(conn: &mut WebSocketConnection) -> Result<()> {
    // sender.send(in)

    // // TODO: Move to GameTask
    // let mut send_task = tokio::spawn(async move {
    //     let n_msg = 20;
    //     for _ in 0..n_msg {
    //         if let Ok(m) = into_ws_msg(&ServerMessage::Connect(ServerConnectMessage {
    //             greeting: String::from("imma server"),
    //             value: 69,
    //         })) {
    //             if sender.send(m).await.is_err() {
    //                 return;
    //             }
    //         }

    //         tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    //     }

    //     if let Err(e) = sender
    //         .send(Message::Close(Some(CloseFrame {
    //             code: axum::extract::ws::close_code::NORMAL,
    //             reason: Utf8Bytes::from_static("Goodbye"),
    //         })))
    //         .await
    //     {
    //         log::debug!("Could not send Close due to {e}, probably it is ok?");
    //     }
    // });

    // let mut recv_task = tokio::spawn(async move {
    //     while let Some(Ok(msg)) = receiver.next().await {
    //         match process_message(msg, who) {
    //             ControlFlow::Continue(Some(m)) => handle_client_message(m),
    //             ControlFlow::Break(_) => break,
    //             _ => {}
    //         }
    //     }
    // });

    // tokio::select! {
    //     r = (&mut send_task) => {
    //         if let Err(e) = r {
    //             log::info!("Error sending messages {e:?}")

    //         }
    //         recv_task.abort();
    //     },
    //     r = (&mut recv_task) => {
    //         if let Err(e) = r {
    //             log::info!("Error receiving messages {e:?}");
    //         }
    //         send_task.abort();
    //     }
    // }

    // TODO: figure out how to deal with receive concurrently...
    // Can split the stream and handle in separate task, but might be difficult
    // to sync if we need to mutex the whole game state..
    // Should split anyway and post parsed messages on channel => then can poll channel

    let mut last_time = Instant::now();
    let mut i = 0;
    loop {
        i += 1;
        conn.send(&ServerMessage::Update(UpdateMessage { value: i }))
            .await?;

        // match conn.receive().await? {
        //     ControlFlow::Continue(_) => todo!(),
        //     ControlFlow::Break(_) => todo!(),
        // }

        let duration = last_time.elapsed();
        last_time = Instant::now();
        if duration < LOOP_MIN_PERIOD {
            tokio::time::sleep(LOOP_MIN_PERIOD - duration).await;
        }
    }

    Ok(())
}

fn handle_client_message(msg: ClientMessage) {
    match msg {
        ClientMessage::Heartbeat => {}
        ClientMessage::Test(m) => {
            log::info!("Test: {:?}", m)
        }
        _ => {
            log::info!("Received unexpected message: {:?}", msg)
        }
    }
}
