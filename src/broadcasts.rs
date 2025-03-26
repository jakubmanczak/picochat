use std::fmt::{Display, Formatter, Result};

use crate::state::User;

#[derive(Debug, Clone)]
pub enum Broadcast {
    UserJoined(User),
    UserLeft(User),
    UserMessage { user: User, message: String },
    UserNickChange { user: User, newname: String },
    UserPoke { poker: User, poked: User },
    UserMe { user: User, message: String },
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
            B::UserPoke { poker, poked } => {
                write!(f, "* {} was poked by {}.\n", poked.name, poker.name)
            }
            B::UserMe { user, message } => {
                write!(f, "* {} {message}\n", user.name)
            }
        }
    }
}

impl Broadcast {
    pub fn send_to_all(&self) -> bool {
        use Broadcast as B;
        match self {
            B::UserJoined(_)
            | B::UserLeft(_)
            | B::UserMessage { .. }
            | B::UserNickChange { .. }
            | B::UserMe { .. } => true,
            B::UserPoke { .. } => false,
        }
    }
    pub fn actor_string(&self) -> String {
        use Broadcast as B;
        match self {
            B::UserPoke { poked, .. } => format!("* You poked {}.\n", poked.name),
            b => b.to_string(),
        }
    }
    pub fn target_string(&self) -> String {
        use Broadcast as B;
        match self {
            B::UserPoke { poker, .. } => format!("* You were poked by {}.\n", poker.name),
            b => b.to_string(),
        }
    }
}
