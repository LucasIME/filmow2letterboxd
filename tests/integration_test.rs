#[cfg(test)]
mod tests {

    use std::{fs::File, io::Read};

    #[tokio::test]
    async fn outputs_azael_profile_correctly() {
        filmow2letterboxd::run("azael".to_string()).await;

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
