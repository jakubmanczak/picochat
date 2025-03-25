use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    broadcasts::Broadcast,
    messages::{REACHED, WELCOME},
    state::{ServerState, User},
};

pub async fn prejoin(socket: &mut TcpStream, state: &ServerState) -> io::Result<()> {
    socket.write_all(REACHED.as_bytes()).await?;
    let userlist = state.list_users().await;
    socket.write_all(userlist.as_bytes()).await?;
    Ok(())
}

pub async fn postjoin(socket: &mut TcpStream) -> io::Result<()> {
    socket.write_all(WELCOME.as_bytes()).await?;
    Ok(())
}

const PICK_NAME: &[u8] = b"Please pick a nickname (ASCII alphanumerics/dots/hyphens only): ";
const NAME_NOT_EMPTY: &[u8] = b"Nickname must not be empty.\n";
const NAME_NOT_PUNCTUATION: &[u8] = b"Nickname must include some letters\n";
const NAME_IN_USE: &[u8] = b"This nickname is already in use.\n";
const CHOSEN_NAME: &str = "You have chosen the nickname: REPL\n";

pub async fn get_nickname(socket: &mut TcpStream, state: &ServerState) -> io::Result<Option<User>> {
    let mut buffer = [0u8; 12];
    loop {
        socket.write_all(PICK_NAME).await?;
        match socket.read(&mut buffer).await {
            Ok(0) | Err(_) => return Ok(None),
            Ok(_) => (),
        };

        let name: String = String::from_utf8_lossy(&buffer)
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
            .collect();

        if name.len() == 0 {
            socket.write_all(NAME_NOT_EMPTY).await?;
            continue;
        }
        if !name.chars().any(|c| c.is_alphabetic()) {
            socket.write_all(NAME_NOT_PUNCTUATION).await?;
            continue;
        }

        let msg = CHOSEN_NAME.replace("REPL", &name);
        socket.write_all(msg.as_bytes()).await?;

        let nickname_in_use = state.is_nickname_in_use(&name).await;
        if !nickname_in_use {
            let user = User { name };
            state.add_user(user.clone()).await;
            state
                .broadcasts
                .send(Broadcast::UserJoined(user.clone()))
                .unwrap();
            return Ok(Some(user));
        }
        socket.write_all(NAME_IN_USE).await?;
        buffer.fill(0);
    }
}
