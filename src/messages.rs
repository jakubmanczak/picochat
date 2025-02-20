pub const REACHED: &str = "You've reached a picochat server.\n";
pub const WELCOME: &str = "You are now connected to the chat server. Messages starting with / are commands and won't be broadcast. Type /help for more info.\n";
pub const USERS_ONLINE: &str = "Users online: REPL.\n";
pub const NO_USERS_ONLINE: &str = "No users online.\n";

pub fn newlinize(string: &str) -> String {
    format!("{string}\n")
}
