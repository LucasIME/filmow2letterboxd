use std::env;
use std::io;
use std::io::prelude::*;
use std::sync::Arc;

mod clients;
use clients::filmow_client::FilmowClient;

mod extractors;
mod model;

mod persisters;
use persisters::csv_writer::CsvWriter;

mod fetchers;

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
    let user = Arc::new(get_username());

    let movies_handle = tokio::spawn(fetch_and_save_movies(user.clone()));
    let watchlist_handle = tokio::spawn(fetch_and_save_watchlist(user.clone()));

    movies_handle
        .await
        .expect("Error while fetching watched movie list");
    watchlist_handle
        .await
        .expect("Error while fetching watchlist");

    println!(
        "Filmow2letterboxed has finished importing your Filmow profile! \
         You should be able to find .csv files in the same directory of the executable. \
         For more instructions on how to import these files to letterboxd, \
         go to https://github.com/LucasIME/filmow2letterboxd"
    );
}

async fn fetch_and_save_movies(user: Arc<String>) {
    let watched_movies_file_name = "watched.csv";
    let watched_movies = FilmowClient::get_all_watched_movies(user).await;

    match CsvWriter::save_movies_to_csv(watched_movies, watched_movies_file_name) {
        Err(e) => return println!("Error when saving watched movies: {:?}", e),
        _ => println!(
            "Successfully generated watched movies file: {}",
            watched_movies_file_name
        ),
    }
}

async fn fetch_and_save_watchlist(user: Arc<String>) {
    let watchlist_file_name = "watchlist.csv";
    let watchlist_movies = FilmowClient::get_all_movies_from_watchlist(user).await;

    match CsvWriter::save_movies_to_csv(watchlist_movies, watchlist_file_name) {
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => println!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }
}
