use csv::Writer;
use std::env;
use std::io;
use std::io::prelude::*;

mod filmowclient;
use filmowclient::FilmowClient;
use filmowclient::Movie;

fn save_movies_to_csv(movies: Vec<Movie>, file_name: &str) -> Result<(), String> {
    let mut wrt = Writer::from_path(file_name)
        .expect(format!("Could not create CSV Writer for file {}", file_name).as_str());
    match wrt.write_record(&["Title", "Directors", "Year"]) {
        Err(e) => {
            return Err(format!(
                "Error when adding header to Csv file {}. {:?}",
                file_name, e
            ))
        }
        _ => {}
    }
    for movie in movies.iter() {
        match wrt.write_record(movie.to_csvable_array()) {
            Err(e) => {
                return Err(format!(
                    "Error when adding entry to Csv file {}. Entry: {:?}, Error:{:?}",
                    file_name, movie, e
                ))
            }
            _ => {}
        }
    }
    match wrt.flush() {
        Err(e) => return Err(format!("Error when flushing file {}. {:?}", file_name, e)),
        _ => {}
    };
    Ok(())
}

fn main() {
    let user = match env::args().nth(1) {
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

    let client = FilmowClient::new();
    let watchlist_file_name = "watchlist.csv";
    let watched_movies_file_name = "watched.csv";

    let watchlist_movies = client.get_all_movies_from_watchlist(user.as_str());
    match save_movies_to_csv(watchlist_movies, watchlist_file_name) {
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => println!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }

    let watched_movies = client.get_all_watched_movies(user.as_str());
    match save_movies_to_csv(watched_movies, watched_movies_file_name) {
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
