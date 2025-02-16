use std::sync::Arc;

use broadcasts::Broadcast;
use messages::{CHOSEN_NAME, PICK_NAME, REACHED, WELCOME};
use state::{ServerState, User};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
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
                println!("{broadcast}");
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
                socket.read(&mut buffer).await.unwrap();
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

            loop {
                // let a = state.
                let mut buffer = [0u8; 256];
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    break;
                }
            }

            // drop the user
            let mut users = state.users.write().await;
            users.retain(|u| u.name != user.name);
            state.broadcasts.send(Broadcast::UserLeft(user)).unwrap();
        });
    }
}
