use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

use crate::{
    broadcasts::Broadcast,
    messages::{CHOSEN_NAME, PICK_NAME, REACHED, WELCOME},
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

pub async fn get_nickname(socket: &mut TcpStream, state: &ServerState) -> io::Result<Option<User>> {
    let mut buffer = [0u8; 12];
    loop {
        socket.write_all(PICK_NAME.as_bytes()).await?;
        match socket.read(&mut buffer).await {
            Ok(0) | Err(_) => return Ok(None),
            Ok(_) => (),
        };
        let name: String = String::from_utf8_lossy(&buffer)
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '.')
            .collect();
        let msg = CHOSEN_NAME.replace("REPL", &name);
        socket.write_all(msg.as_bytes()).await?;

        let nickname_in_use = state.users.read().await.iter().any(|u| u.name == name);
        if !nickname_in_use {
            let user = User { name };
            let mut users = state.users.write().await;
            users.push(user.clone());
            state
                .broadcasts
                .send(Broadcast::UserJoined(user.clone()))
                .unwrap();
            return Ok(Some(user));
        }
    }
}
