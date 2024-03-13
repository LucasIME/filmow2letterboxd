use std::{env, io, io::prelude::*};

mod logging;

fn get_username() -> String {
    match env::args().nth(1) {
        None => {
            print!("Please, enter the your Filmow username: ");
            io::stdout().flush().expect("could not flush stdout");
            let mut user_input = String::new();
            io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read user input");
            user_input
        }
        Some(user) => user,
    }
}

#[tokio::main]
async fn main() {
    logging::setup_logging();

    filmow2letterboxd::run(get_username()).await;
}
