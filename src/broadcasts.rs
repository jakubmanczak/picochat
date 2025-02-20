use std::fmt::{Display, Formatter, Result};

use crate::state::User;

#[derive(Debug, Clone)]
pub enum Broadcast {
    UserJoined(User),
    UserLeft(User),
    UserMessage { user: User, message: String },
    UserNickChange { user: User, newname: String },
}

impl Display for Broadcast {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use Broadcast as B;
        match self {
            B::UserJoined(u) => write!(f, "* {} joined the chat.\n", u.name),
            B::UserLeft(u) => write!(f, "* {} left the chat.\n", u.name),
            B::UserMessage { user, message } => write!(f, "{}: {message}\n", user.name),
            B::UserNickChange { user, newname } => write!(
                f,
                "* {} changed their nickname to {}.\n",
                user.name, newname
            ),
        }
    }
}
