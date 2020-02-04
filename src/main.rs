use csv::Writer;
use std::env;

mod filmowclient;
use filmowclient::FilmowClient;
use filmowclient::Movie;

fn save_movies_to_csv(movies: Vec<Movie>, file_name: &str) {
    let mut wrt = Writer::from_path(file_name).unwrap();
    wrt.write_record(&["Title", "Directors", "Year"]);
    for movie in movies.iter() {
        wrt.write_record(movie.to_csvable_array());
    }
    wrt.flush();
}

fn main() {
    let user = env::args().nth(1).unwrap();
    let client = FilmowClient::new();
    let watchlist_movies = client.get_all_movies_from_watchlist(user.as_str());
    save_movies_to_csv(watchlist_movies, "watchlist.csv");
    let watched_movies = client.get_all_watched_movies(user.as_str());
    save_movies_to_csv(watched_movies, "watched.csv");
}
