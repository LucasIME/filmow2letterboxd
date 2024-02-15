use crate::extractors::movie_extractor::MovieExtractor;

use crate::model::movie::Movie;
use tokio_retry::{
    strategy::{jitter, ExponentialBackoff},
    Retry,
};

use std::sync::Arc;

use crate::fetchers::{
    watched_list_fetcher::WatchedMoviesFetcher, watchlist_fetcher::WatchlistFetcher,
};

use reqwest::{header, Client};

#[derive(Debug, Clone)]
pub struct FilmowClient {
    client: Client,
}

impl FilmowClient {
    pub fn new() -> Self {
        let user_agent = header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3");
        Self {
            client: Client::builder().user_agent(user_agent).build().unwrap(),
        }
    }

    pub async fn get_all_movies_from_watchlist(&self, user: Arc<String>) -> Vec<Movie> {
        let watchlist_fetcher = WatchlistFetcher::new(self.clone());
        watchlist_fetcher.get_all_movies_from_watchlist(user).await
    }

    pub async fn get_all_watched_movies(&self, user: Arc<String>) -> Vec<Movie> {
        let watched_list_fetcher = WatchedMoviesFetcher::new(self.clone());
        watched_list_fetcher.get_all_watched_movies(user).await
    }

    pub fn get_base_url() -> String {
        "https://filmow.com".to_string()
    }

    pub async fn get_html_from_url(&self, url: &str) -> Result<String, String> {
        let retry_strategy = ExponentialBackoff::from_millis(10).map(jitter).take(5);
        Retry::spawn(retry_strategy, || async move {
            self.get_html_from_url_no_retry(url).await
        })
        .await
    }

    async fn get_html_from_url_no_retry(&self, url: &str) -> Result<String, String> {
        match self.client.get(url).send().await {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found".to_string());
                }
                match resp.text().await {
                    Ok(text) => Ok(text),
                    Err(e) => Err(format!(
                        "Failed to get text form url {}. Error was {}",
                        url, e
                    )),
                }
            }
            Err(e) => Err(format!(
                "Failed to get HTML for url: {}. Received error: {:?}",
                url, e
            )),
        }
    }

    async fn get_movie_from_url(&self, url: &str) -> Result<Movie, String> {
        match self.get_html_from_url(url).await {
            Ok(html_body) => MovieExtractor::extract_movie_from_html(html_body.as_str(), url),
            Err(e) => Err(e),
        }
    }

    pub async fn parallel_build_movie_from_preliminary_info(
        &self,
        info_vec: Vec<PreliminaryMovieInformation>,
    ) -> Vec<Movie> {
        let mut children = vec![];

        for info in info_vec {
            let self_clone = self.clone();
            children.push(
                tokio::spawn(async move {
                println!("Fetching information for movie {}", info.movie_url);
                match self_clone.get_movie_from_url(info.movie_url.as_str()).await {
                        Ok(movie) => {
                            println!("Successfully fetched information for Movie {}", movie.title);
                            Some(Movie {
                                title: movie.title,
                                director: movie.director,
                                year: movie.year,
                                rating: info.rating
                            })
                        }
                        Err(e) => {
                            println!("Could not construct movie from url {}. Ignoring it and continuing. Error was: {}", info.movie_url, e);
                            return None;
                        }
                    }
            }));
        }

        let mut movies = vec![];
        for child in children {
            let movie = child.await.expect("Could not join child thread");
            movies.push(movie);
        }

        movies.into_iter().flatten().collect()
    }
}

#[derive(Debug)]
pub struct PreliminaryMovieInformation {
    pub movie_url: String,
    pub rating: Option<f32>,
}
