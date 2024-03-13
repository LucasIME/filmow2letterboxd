use std::sync::Arc;

mod clients;
use clients::filmow_client::FilmowClient;

mod extractors;
mod model;

mod persisters;
use persisters::csv_writer::CsvWriter;

mod fetchers;

pub async fn run(user: String) {
    let filmow_client = Arc::new(FilmowClient::new());
    let user = Arc::new(user);

    let movies_handle = tokio::spawn(fetch_and_save_movies(filmow_client.clone(), user.clone()));
    let watchlist_handle = tokio::spawn(fetch_and_save_watchlist(
        filmow_client.clone(),
        user.clone(),
    ));

    movies_handle
        .await
        .expect("Error while fetching watched movie list");
    watchlist_handle
        .await
        .expect("Error while fetching watchlist");

    log::info!(
        "Filmow2letterboxed has finished importing your Filmow profile! \
         You should be able to find .csv files in the same directory of the executable. \
         For more instructions on how to import these files to letterboxd, \
         go to https://github.com/LucasIME/filmow2letterboxd"
    );
}

async fn fetch_and_save_movies(client: Arc<FilmowClient>, user: Arc<String>) {
    let watched_movies_file_name = "watched.csv";
    let mut watched_movies = FilmowClient::get_all_watched_movies(client, user).await;
    watched_movies.sort_by_key(|movie| movie.title.clone());

    match CsvWriter::save_movies_to_csv(watched_movies, watched_movies_file_name) {
        Err(e) => return log::error!("Error when saving watched movies: {:?}", e),
        _ => log::info!(
            "Successfully generated watched movies file: {}",
            watched_movies_file_name
        ),
    }
}

async fn fetch_and_save_watchlist(client: Arc<FilmowClient>, user: Arc<String>) {
    let watchlist_file_name = "watchlist.csv";
    let mut watchlist_movies = FilmowClient::get_all_movies_from_watchlist(client, user).await;
    watchlist_movies.sort_by_key(|movie| movie.title.clone());

    match CsvWriter::save_movies_to_csv(watchlist_movies, watchlist_file_name) {
        Err(e) => return log::error!("Error when saving watchlist: {:?}", e),
        _ => log::info!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }
}
