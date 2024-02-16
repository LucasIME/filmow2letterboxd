use std::sync::Arc;

use crate::{
    clients::filmow_client::FilmowClient, extractors::movie_extractor::MovieExtractor,
    model::movie::Movie,
};

#[derive(Clone)]
pub struct WatchedMoviesFetcher {
    filmow_client: Arc<FilmowClient>,
}

impl WatchedMoviesFetcher {
    pub fn new(filmow_client: Arc<FilmowClient>) -> Self {
        WatchedMoviesFetcher { filmow_client }
    }

    pub async fn get_all_watched_movies(
        shared_self: Arc<WatchedMoviesFetcher>,
        user: Arc<String>,
    ) -> Vec<Movie> {
        println!("Fetching watched movies for user {}", user);

        let number_of_pages = shared_self.get_last_watched_page_number(user.clone()).await;
        println!("Number of watched movies pages {:?}", number_of_pages);

        let mut resp = vec![];
        let mut handles = vec![];
        for page_num in 1..=number_of_pages {
            let self_clone = shared_self.clone();
            let user_clone = user.clone();

            let page_movies_handle = tokio::spawn(async move {
                self_clone
                    .get_all_movies_for_watched_page(page_num, user_clone)
                    .await
            });
            handles.push(page_movies_handle)
        }

        for handle in handles {
            let mut movies = handle.await.unwrap();
            resp.append(&mut movies);
        }
        resp
    }

    pub async fn get_all_movies_for_watched_page(
        &self,
        page_num: i32,
        user: Arc<String>,
    ) -> Vec<Movie> {
        let watched_url_for_page = Self::get_watched_url_for_page(user, page_num);
        match self
            .filmow_client
            .get_html_from_url(watched_url_for_page.as_str())
            .await
        {
            Ok(watched_page_html) => {
                let preliminary_movies_info =
                    MovieExtractor::get_preliminary_info_for_watched_movies(
                        watched_page_html.as_str(),
                    );
                let page_movies = FilmowClient::parallel_build_movie_from_preliminary_info(
                    self.filmow_client.clone(),
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

    async fn get_last_watched_page_number(&self, user: Arc<String>) -> i32 {
        println!("Getting total number of watched pages");
        let watched_url = Self::get_watched_url_for_page(user, 1);
        match self
            .filmow_client
            .get_html_from_url(watched_url.as_str())
            .await
        {
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
