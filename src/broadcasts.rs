use std::fmt::{Display, Formatter, Result};

use crate::state::User;

#[derive(Debug, Clone)]
pub enum Broadcast {
    UserJoined(User),
    UserLeft(User),
    UserMessage { user: User, message: String },
}

impl Display for Broadcast {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Broadcast as B;
        match self {
            B::UserJoined(u) => write!(f, "{} joined the chat.", u.name),
            B::UserLeft(u) => write!(f, "{} left the chat.", u.name),
            B::UserMessage { user, message } => write!(f, "|{}|: {message}", user.name),
        }
    }
}
