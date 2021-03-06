use std::env;
use std::io;
use std::io::prelude::*;

mod filmowclient;
use filmowclient::FilmowClient;

mod csvwriter;
use csvwriter::CsvWriter;

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
    let user = get_username();
    let user_clone = user.clone();

    let watchlist_file_name = "watchlist.csv";
    let watched_movies_file_name = "watched.csv";

    let watchlist_movies_handle =
        tokio::spawn(
            async move { FilmowClient::get_all_movies_from_watchlist(user.as_str()).await },
        );
    let watched_movies_handle =
        tokio::spawn(
            async move { FilmowClient::get_all_watched_movies(user_clone.as_str()).await },
        );

    match CsvWriter::save_movies_to_csv(watchlist_movies_handle.await.unwrap(), watchlist_file_name)
    {
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => println!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }

    match CsvWriter::save_movies_to_csv(
        watched_movies_handle.await.unwrap(),
        watched_movies_file_name,
    ) {
        Err(e) => return println!("Error when saving watched movies: {:?}", e),
        _ => println!(
            "Successfully generated watched movies file: {}",
            watched_movies_file_name
        ),
    }

    println!(
        "Filmow2letterboxed has finished importing your Filmow profile! \
         You should be able to find .csv files in the same directory of the executable. \
         For more instructions on how to import these files to letterboxd, \
         go to https://github.com/LucasIME/filmow2letterboxd"
    );
}
