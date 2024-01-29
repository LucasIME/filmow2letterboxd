use std::sync::Arc;

use crate::clients::filmow_client::FilmowClient;
use crate::extractors::movie_extractor::MovieExtractor;
use crate::model::movie::Movie;

pub struct WatchedMoviesFetcher {}

impl WatchedMoviesFetcher {
    pub async fn get_all_watched_movies(user: Arc<String>) -> Vec<Movie> {
        println!("Fetching watched movies for user {}", user);

        let number_of_pages = Self::get_last_watched_page_number(user.clone()).await;
        println!("Number of watched movies pages {:?}", number_of_pages);

        let mut resp = vec![];
        let mut handles = vec![];
        for page_num in 1..=number_of_pages {
            let page_movies_handle = tokio::spawn(Self::get_all_movies_for_watched_page(
                page_num,
                user.clone(),
            ));
            handles.push(page_movies_handle)
        }

        for handle in handles {
            let mut movies = handle.await.unwrap();
            resp.append(&mut movies);
        }
        resp
    }

    pub async fn get_all_movies_for_watched_page(page_num: i32, user: Arc<String>) -> Vec<Movie> {
        let watched_url_for_page = Self::get_watched_url_for_page(user, page_num);
        match FilmowClient::get_html_from_url(watched_url_for_page.as_str()).await {
            Ok(watched_page_html) => {
                let preliminary_movies_info =
                    MovieExtractor::get_preliminary_info_for_watched_movies(
                        watched_page_html.as_str(),
                    );
                let page_movies = FilmowClient::parallel_build_movie_from_preliminary_info(
                    preliminary_movies_info,
                )
                .await;
                println!("Movies for watched page {}: {:?}", page_num, page_movies);
                page_movies
            }
            Err(e) => {
                println!(
                    "Failed to get html for url {}. Error: {}",
                    watched_url_for_page, e
                );
                vec![]
            }
        }
    }

    async fn get_last_watched_page_number(user: Arc<String>) -> i32 {
        println!("Getting total number of watched pages");
        let watched_url = Self::get_watched_url_for_page(user, 1);
        match FilmowClient::get_html_from_url(watched_url.as_str()).await {
            Ok(watched_page_html) => {
                MovieExtractor::get_last_page_from_html(watched_page_html.as_str()).unwrap_or(1)
            }
            Err(e) => {
                panic!("Error when trying to find number of watched pages: {}", e);
            }
        }
    }

    fn get_watched_url_for_page(user: Arc<String>, page: i32) -> String {
        if page == 1 {
            return format!("https://filmow.com/usuario/{}/ja-vi/", user);
        }

        format!("https://filmow.com/usuario/{}/ja-vi/?pagina={}", user, page)
    }
}
