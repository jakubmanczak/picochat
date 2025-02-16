use tokio::sync::{
    broadcast::{self, Sender},
    RwLock,
};

use crate::{
    broadcasts::Broadcast,
    messages::{NO_USERS_ONLINE, USERS_ONLINE},
};

#[derive(Debug, Clone)]
pub struct User {
    pub name: String,
}

#[derive(Debug)]
pub struct ServerState {
    pub users: RwLock<Vec<User>>,
    pub broadcasts: Sender<Broadcast>,
}

impl ServerState {
    pub fn new() -> ServerState {
        ServerState {
            users: RwLock::new(Vec::new()),
            broadcasts: broadcast::channel::<Broadcast>(8).0,
        }
    }
    pub async fn list_users(&self) -> String {
        let mut userlist = String::new();
        let users = self.users.read().await;
        if users.len() == 0 {
            return NO_USERS_ONLINE.to_string();
        } else {
            for (i, user) in users.iter().enumerate() {
                if i == 0 {
                    userlist = userlist + &user.name;
                } else {
                    userlist = userlist + ", " + &user.name;
                }
            }
            return USERS_ONLINE.replace("REPL", &userlist);
        }
    }
}
