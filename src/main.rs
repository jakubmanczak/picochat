use broadcasts::Broadcast;
use commands::handle_commands;
use state::ServerState;
use std::{env, sync::Arc};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    spawn,
};

pub mod broadcasts;
pub mod commands;
pub mod messages;
pub mod routines;
pub mod state;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|s| s == "-v" || s == "--version") {
        let ver = env!("CARGO_PKG_VERSION");
        let sha = env!("GIT_SHORTHASH");
        let por = env!("GIT_PORCELAIN");
        let rel = match cfg!(debug_assertions) {
            true => "debug",
            false => "release",
        };
        println!("picochat v{ver} ({sha} {rel} {por})");
        return;
    }

    let listener = TcpListener::bind("0.0.0.0:7426").await.unwrap();
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
            match routines::prejoin(&mut socket, &state).await {
                Ok(_) => (),
                Err(_) => return,
            };
            let mut user = match routines::get_nickname(&mut socket, &state).await {
                Ok(Some(u)) => u,
                Ok(None) | Err(_) => return,
            };
            match routines::postjoin(&mut socket).await {
                Ok(_) => (),
                Err(_) => return,
            };

            let (mut rsocket, mut wsocket) = io::split(socket);

            let mut rx = state.broadcasts.subscribe();
            let mut buffer = [0u8; 256];
            loop {
                buffer.fill(0);
                tokio::select! {
                    res = rx.recv() => {
                        let res = res.unwrap();
                        match res.send_to_all() {
                            true => {
                                let res = res.to_string();
                                match wsocket.write_all(res.as_bytes()).await {
                                    Ok(_) => (),
                                    Err(_) => break,
                                }
                            }
                            false => {
                                if let Broadcast::UserPoke{poker, poked} = &res {
                                    if &user.name == &poker.name {
                                        let res = res.actor_string();
                                        match wsocket.write_all(res.as_bytes()).await {
                                            Ok(_) => (),
                                            Err(_) => break,
                                        }
                                    }
                                    if &user.name == &poked.name {
                                        let res = res.target_string();
                                        match wsocket.write_all(res.as_bytes()).await {
                                            Ok(_) => (),
                                            Err(_) => break,
                                        }
                                    }
                                }
                            }
                        }
                    }
                    res = rsocket.read(&mut buffer) => {
                        match res {
                            Ok(0) | Err(_) => break,
                            Ok(_) => {
                                let msg: String = String::from_utf8_lossy(&buffer).chars()
                                    .filter(|c| {
                                         c.is_alphabetic() || c.is_digit(10) || c.is_ascii_punctuation() || *c == ' '
                                    }).collect();

                                match msg.starts_with('/') {
                                    true => {
                                        match handle_commands(msg, &mut wsocket, &mut user, &state).await {
                                            Ok(_) => (),
                                            Err(_) => return,
                                        }
                                    },
                                    false => {
                                        state.broadcasts.send(Broadcast::UserMessage {
                                            user: user.clone(),
                                            message: msg,
                                        }).unwrap();
                                    }
                                }
                            },
                        }
                    }
                }
            }

            state.remove_user(&user).await;
            state.broadcasts.send(Broadcast::UserLeft(user)).unwrap();
        });
    }
}
