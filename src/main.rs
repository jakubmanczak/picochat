use std::sync::Arc;

use broadcasts::Broadcast;
use messages::{CHOSEN_NAME, PICK_NAME, REACHED, WELCOME};
use state::{ServerState, User};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    spawn,
};

pub mod broadcasts;
pub mod messages;
pub mod state;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:2004").await.unwrap();
    let state = Arc::new(ServerState::new());

    {
        let state = state.clone();
        spawn(async move {
            let mut rx = state.broadcasts.subscribe();
            loop {
                let broadcast = rx.recv().await.unwrap();
                print!("{broadcast}");
            }
        });
    }

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();
        let state = state.clone();
        spawn(async move {
            socket.write_all(REACHED.as_bytes()).await.unwrap();
            let userlist = state.list_users().await;
            socket.write_all(userlist.as_bytes()).await.unwrap();

            let user;
            loop {
                socket.write_all(PICK_NAME.as_bytes()).await.unwrap();
                let mut buffer = [0u8; 12];
                match socket.read(&mut buffer).await {
                    Ok(0) | Err(_) => return,
                    Ok(_) => (),
                };
                let name: String = String::from_utf8_lossy(&buffer)
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
                    .collect();
                let msg = CHOSEN_NAME.replace("REPL", &name);
                socket.write_all(msg.as_bytes()).await.unwrap();

                let users = state.users.read().await;
                if !users.iter().any(|u| u.name == name) {
                    drop(users);
                    user = User { name };
                    let mut users = state.users.write().await;
                    users.push(user.clone());
                    state
                        .broadcasts
                        .send(Broadcast::UserJoined(user.clone()))
                        .unwrap();
                    break;
                }
            }
            socket.write_all(WELCOME.as_bytes()).await.unwrap();

            let (mut rsocket, mut wsocket) = io::split(socket);

            let mut buffer = [0u8; 256];
            let mut rx = state.broadcasts.subscribe();
            loop {
                tokio::select! {
                    res = rx.recv() => {
                        let res = res.unwrap().to_string();
                        match wsocket.write_all(res.as_bytes()).await {
                            Ok(_) => (),
                            Err(_) => break,
                        }
                    }
                    res = rsocket.read(&mut buffer) => {
                        match res {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                state.broadcasts.send(Broadcast::UserMessage {
                                    user: user.clone(),
                                    message: String::from_utf8_lossy(&buffer).chars()
                                        .filter(|c| {
                                             c.is_alphabetic() || c.is_digit(10) || c.is_ascii_punctuation() || *c == ' '
                                        }).collect::<String>(),
                                }).unwrap();
                            },
                        }
                    }
                }
            }

            let mut users = state.users.write().await;
            users.retain(|u| u.name != user.name);
            state.broadcasts.send(Broadcast::UserLeft(user)).unwrap();
        });
    }
}
