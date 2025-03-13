use std::io::{self, Write};

pub fn ask_user(prompt: &str) -> bool {
    print!("{}", prompt);
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap_or_default();

    response.trim().to_lowercase() == "y"
}
