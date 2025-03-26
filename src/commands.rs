use std::collections::VecDeque;

use tokio::{
    io::{self, AsyncWriteExt, WriteHalf},
    net::TcpStream,
};

use crate::{
    broadcasts::Broadcast,
    state::{ServerState, User},
};

const CMD_BAD: &[u8] = b"* Command doesn't exist or wrong number or args supplied.\n";

const NAME_IN_USE: &[u8] = b"* This nickname is already in use.\n";
const NAME_NOT_EMPTY: &[u8] = b"* Nickname must not be empty.\n";
const NAME_NOT_PUNCTUATION: &[u8] = b"* Nickname must include some letters\n";
const NAME_UNDER_12CHAR: &[u8] = b"* Nickname must be at most 12 characters long.\n";
const NAME_INVALID_CHARS: &[u8] = b"* Nickname must only be ASCII alphanumerics/dots/hyphens.\n";

const HELP: &str = r#"* Picochat is a chat program that works over TCP.
* Messages starting with / are commands and are not broadcasted. Available commands:
* |---------------------------------------------------------------------|
* | /ping  | 0 arguments | A test command to check TCP connection.      |
* | /help  | 0 arguments | Displays this help menu.                     |
* | /users | 0 arguments | Shows the list of online users.              |
* | /nick  | 0 arguments | Shows your current nickname.                 |
* | /nick  | 1 argument  | Changes your nickname (broadcast to others). |
* | /poke  | 1 argument  | Poke somebody (broadcast only to them).      |
* | /me    | n arguments | Broadcasts chosen action in third person.    |
* | /echo  | n arguments | Prints back your arguments.                  |
* |---------------------------------------------------------------------|
"#;

pub async fn handle_commands(
    msg: String,
    wsocket: &mut WriteHalf<TcpStream>,
    user: &mut User,
    state: &ServerState,
) -> io::Result<()> {
    let mut parts: VecDeque<&str> = msg.split(' ').collect();
    let command = parts.pop_front().unwrap();
    let argcount = parts.len();
    parts.retain(|e| *e != " ");

    match (command, argcount, parts) {
        ("/ping", 0, _) => wsocket.write_all(b"* Pong!\n").await?,
        ("/help", 0, _) => wsocket.write_all(HELP.as_bytes()).await?,
        ("/users", 0, _) => {
            let list = format!("* {}", state.list_users().await);
            wsocket.write_all(list.as_bytes()).await?;
        }
        ("/nick", 0, _) => {
            wsocket
                .write_all(format!("* Your nickname is {}\n", user.name).as_bytes())
                .await?;
        }
        ("/nick", 1, parts) => {
            if parts[0].len() == 0 {
                wsocket.write_all(NAME_NOT_EMPTY).await?;
                return Ok(());
            }
            if !parts[0].chars().any(|c| c.is_alphabetic()) {
                wsocket.write_all(NAME_NOT_PUNCTUATION).await?;
                return Ok(());
            }
            if parts[0].len() > 12 {
                wsocket.write_all(NAME_UNDER_12CHAR).await?;
                return Ok(());
            }
            if parts[0]
                .chars()
                .any(|c| !c.is_ascii_alphanumeric() && c != '-' && c != '.')
            {
                wsocket.write_all(NAME_INVALID_CHARS).await?;
                return Ok(());
            }

            let nickname_in_use = state.is_nickname_in_use(parts[0]).await;
            if nickname_in_use {
                wsocket.write_all(NAME_IN_USE).await?;
                return Ok(());
            }

            state
                .broadcasts
                .send(Broadcast::UserNickChange {
                    user: user.clone(),
                    newname: parts[0].to_string(),
                })
                .unwrap();
            state.change_nickname(&user.name, parts[0]).await;
            user.name = parts[0].to_string();
        }
        ("/poke", 1, parts) => {
            let target = parts[0];
            if user.name == target {
                wsocket.write_all(b"* You poked yourself.\n").await?;
            } else {
                let users = state.users.read().await;
                if users.iter().any(|u| u.name == target) {
                    state
                        .broadcasts
                        .send(Broadcast::UserPoke {
                            poker: user.clone(),
                            poked: users
                                .iter()
                                .filter(|u| u.name == target)
                                .next()
                                .unwrap()
                                .clone(),
                        })
                        .unwrap();
                } else {
                    wsocket.write_all(b"* No such user found.\n").await?;
                }
            }
        }
        ("/echo", _, parts) => {
            wsocket
                .write_all(format!("* Echo: {:?}\n", parts).as_bytes())
                .await?
        }
        ("/me", 1.., parts) => {
            let b = Broadcast::UserMe {
                user: user.clone(),
                message: parts
                    .iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>()
                    .join(" "),
            };
            state.broadcasts.send(b).unwrap();
        }
        (_, _, _) => wsocket.write_all(CMD_BAD).await?,
    }

    Ok(())
}
