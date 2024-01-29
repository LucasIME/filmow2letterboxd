use std::sync::Arc;

use crate::clients::filmow_client::FilmowClient;
use crate::extractors::movie_extractor::MovieExtractor;
use crate::model::movie::Movie;

pub struct WatchlistFetcher {}

impl WatchlistFetcher {
    pub async fn get_all_movies_from_watchlist(user: Arc<String>) -> Vec<Movie> {
        println!("Fetching watchlist for user {}", user);

        let number_of_pages = Self::get_last_watchlist_page_number(user.clone()).await;
        println!("Number of watchlist pages {:?}", number_of_pages);

        let mut resp = vec![];
        let mut handles = vec![];
        for page_num in 1..=number_of_pages {
            let page_movies_handle = tokio::spawn(Self::get_all_movies_for_watchlist_page(
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

    pub async fn get_all_movies_for_watchlist_page(page_num: i32, user: Arc<String>) -> Vec<Movie> {
        println!("Processing watched movies page {}", page_num);

        let watchlist_url = Self::get_watchlist_url_for_page(user, page_num);
        match FilmowClient::get_html_from_url(watchlist_url.as_str()).await {
            Ok(watchlist_page_html) => {
                let preliminary_movies_info = MovieExtractor::get_preliminary_info_for_watchlist(
                    watchlist_page_html.as_str(),
                );
                let page_movies = FilmowClient::parallel_build_movie_from_preliminary_info(
                    preliminary_movies_info,
                )
                .await;
                println!("Movies for watchlist page {}: {:?}", page_num, page_movies);
                page_movies
            }
            _ => {
                print!("Error fetching watchlist for page {}", page_num);
                vec![]
            }
        }
    }

    async fn get_last_watchlist_page_number(user: Arc<String>) -> i32 {
        println!("Getting total number of watchlist pages");
        let watchlist_url = Self::get_watchlist_url_for_page(user, 1);
        match FilmowClient::get_html_from_url(watchlist_url.as_str()).await {
            Ok(watchlist_page_html) => {
                MovieExtractor::get_last_page_from_html(watchlist_page_html.as_str()).unwrap_or(1)
            }
            Err(e) => {
                panic!("Error when trying to find number of watchlist pages: {}", e);
            }
        }
    }

    fn get_watchlist_url_for_page(user: Arc<String>, page: i32) -> String {
        if page == 1 {
            return format!("https://filmow.com/usuario/{}/quero-ver/", user);
        }

        format!(
            "https://filmow.com/usuario/{}/quero-ver/?pagina={}",
            user, page
        )
    }
}
