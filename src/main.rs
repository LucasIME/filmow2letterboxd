use csv::Writer;
use std::env;

mod filmowclient;
use filmowclient::FilmowClient;
use filmowclient::Movie;

fn save_movies_to_csv(movies: Vec<Movie>, file_name: &str) -> Result<(), String> {
    let mut wrt = Writer::from_path(file_name).unwrap();
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
    let user = env::args().nth(1).unwrap();
    let client = FilmowClient::new();
    let watchlist_movies = client.get_all_movies_from_watchlist(user.as_str());
    match save_movies_to_csv(watchlist_movies, "watchlist.csv") {
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => {}
    }
    let watched_movies = client.get_all_watched_movies(user.as_str());
    match save_movies_to_csv(watched_movies, "watched.csv") {
        Err(e) => return println!("Error when saving watched movies: {:?}", e),
        _ => {}
    }
}
