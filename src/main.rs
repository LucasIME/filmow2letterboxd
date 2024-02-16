use std::{env, io, io::prelude::*, sync::Arc};

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
    let filmow_client = Arc::new(FilmowClient::new());
    let user = Arc::new(get_username());

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

    println!(
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
        Err(e) => return println!("Error when saving watched movies: {:?}", e),
        _ => println!(
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
        Err(e) => return println!("Error when saving watchlist: {:?}", e),
        _ => println!(
            "Successfully generated watchlist file: {}",
            watchlist_file_name
        ),
    }
}

#[cfg(test)]
mod tests {

    use std::{fs::File, io::Read, process::Command};

    #[test]
    fn outputs_azael_profile_correctly() {
        let output = Command::new("cargo")
            .args(&["run", "azael"])
            .output()
            .expect("failed to execute process");

        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        println!("stdout: {}", stdout);

        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        println!("stderr: {}", stderr);

        let expected_watchlist_content =
            get_file_content("./tests/resources/expected_watchlist_azael.csv");
        let watchlist_content = get_file_content("./watchlist.csv");

        let expected_watched_list_content =
            get_file_content("./tests/resources/expected_watched_list_azael.csv");
        let watched_list_content = get_file_content("./watched.csv");

        assert_eq!(watchlist_content, expected_watchlist_content);
        assert_eq!(watched_list_content, expected_watched_list_content);
    }

    fn get_file_content(file_path: &str) -> String {
        let mut file = match File::open(file_path) {
            Ok(file) => file,
            Err(e) => panic!("Error opening expected watchlist file: {}", e),
        };

        let mut content = String::new();
        if let Err(e) = file.read_to_string(&mut content) {
            eprintln!("Error reading the file: {}", e);
            panic!("Failed to read file");
        }

        return content;
    }
}
