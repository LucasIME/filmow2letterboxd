use std::env;
use std::io;
use std::io::prelude::*;

mod filmowclient;
use filmowclient::FilmowClient;

mod csvwriter;
use csvwriter::CsvWriter;

fn get_username() -> String {
    return match env::args().nth(1) {
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
    };
}

#[tokio::main]
async fn main() {
    let user = get_username();

    let watchlist_file_name = "watchlist.csv";
    let watched_movies_file_name = "watched.csv";

    let watchlist_movies = FilmowClient::get_all_movies_from_watchlist(user.as_str());
    match CsvWriter::save_movies_to_csv(watchlist_movies.await, watchlist_file_name) {
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => println!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }

    let watched_movies = FilmowClient::get_all_watched_movies(user.as_str());
    match CsvWriter::save_movies_to_csv(watched_movies.await, watched_movies_file_name) {
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
