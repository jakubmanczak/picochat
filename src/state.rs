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
    pub async fn add_user(&self, user: User) {
        let mut users = self.users.write().await;
        users.push(user);
    }
    pub async fn remove_user(&self, user: &User) {
        let mut users = self.users.write().await;
        users.retain(|u| u.name != user.name);
    }
    pub async fn is_nickname_in_use(&self, nickname: &str) -> bool {
        let users = self.users.read().await;
        users.iter().any(|u| u.name == nickname)
    }
    pub async fn change_nickname(&self, old_name: &str, new_name: &str) {
        let mut users = self.users.write().await;
        if let Some(user) = users.iter_mut().find(|u| u.name == old_name) {
            user.name = new_name.to_string();
        }
    }
    pub async fn list_users(&self) -> String {
        let userlist = self.users.read().await;
        if userlist.is_empty() {
            NO_USERS_ONLINE.to_string()
        } else {
            let userstring = userlist
                .iter()
                .map(|u| u.name.clone())
                .collect::<Vec<_>>()
                .join(", ");
            USERS_ONLINE.replace("REPL", &userstring)
        }
    }
}
