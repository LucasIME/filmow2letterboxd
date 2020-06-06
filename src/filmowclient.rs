use reqwest;

mod movieextractor;
use movieextractor::MovieExtractor;

pub mod movie;
use movie::Movie;

#[derive(Debug)]
pub struct FilmowClient {}

impl FilmowClient {
    pub async fn get_all_movies_from_watchlist(user: &str) -> Vec<Movie> {
        println!("Fetching watchlist for user {}", user);
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            let watchlist_url = FilmowClient::get_watchlist_url_for_page(user, page_num);
            match FilmowClient::get_html_from_url(watchlist_url.as_str()).await {
                Ok(watchlist_page_html) => {
                    let preliminary_movies_info =
                        MovieExtractor::get_preliminary_info_for_watchlist(
                            watchlist_page_html.as_str(),
                        );
                    let mut page_movies = FilmowClient::parallel_build_movie_from_preliminary_info(
                        preliminary_movies_info,
                    )
                    .await;
                    println!("Movies for page {}: {:?}", page_num, page_movies);
                    resp.append(&mut page_movies);
                    page_num += 1;
                }
                _ => break,
            }
        }

        return resp;
    }

    pub async fn get_all_watched_movies(user: &str) -> Vec<Movie> {
        println!("Fetching watched movies for user {}", user);
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            let watched_url_for_page = FilmowClient::get_watched_url_for_page(user, page_num);
            match FilmowClient::get_html_from_url(watched_url_for_page.as_str()).await {
                Ok(watched_page_html) => {
                    let preliminary_movies_info =
                        MovieExtractor::get_preliminary_info_for_watched_movies(
                            watched_page_html.as_str(),
                        );
                    let mut page_movies = FilmowClient::parallel_build_movie_from_preliminary_info(
                        preliminary_movies_info,
                    )
                    .await;
                    println!("Movies for page {}: {:?}", page_num, page_movies);
                    resp.append(&mut page_movies);
                    page_num += 1;
                }
                Err(e) => {
                    println!(
                        "Failed to get html for url {}. Error: {}",
                        watched_url_for_page, e
                    );
                    break;
                }
            }
        }
        return resp;
    }

    fn get_base_url() -> String {
        "https://filmow.com".to_string()
    }

    fn get_watchlist_url_for_page(user: &str, page: i32) -> String {
        format!(
            "https://filmow.com/usuario/{}/filmes/quero-ver/?pagina={}",
            user, page
        )
    }

    fn get_watched_url_for_page(user: &str, page: i32) -> String {
        format!(
            "https://filmow.com/usuario/{}/filmes/ja-vi/?pagina={}",
            user, page
        )
    }

    async fn get_html_from_url(url: &str) -> Result<String, String> {
        println!("Getting HTML for url: {}", url);
        match reqwest::get(url).await {
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
            Err(e) => {
                return Err(format!(
                    "Failed to get HTML for url: {}. Reived error: {:?}",
                    url, e
                ));
            }
        }
    }

    async fn get_movie_from_url(url: &str) -> Result<Movie, String> {
        match reqwest::get(url).await {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err(format!("404 page not found, when fetching for url {}", url));
                }

                let html_body = match resp.text().await {
                    Ok(body) => body,
                    Err(e) => {
                        return Err(format!(
                            "Error when getting html body for url {}. Error: {:?}",
                            url, e
                        ));
                    }
                };

                return MovieExtractor::extract_movie_from_html(html_body.as_str(), url);
            }
            Err(e) => Err(format!(
                "Reqwest error when fetching url {}. Error: {:?}",
                url, e
            )),
        }
    }

    async fn parallel_build_movie_from_preliminary_info(
        info_vec: Vec<PreliminaryMovieInformation>,
    ) -> Vec<Movie> {
        let mut children = vec![];
        for info in info_vec {
            children.push(
                tokio::spawn(async move {
                match FilmowClient::get_movie_from_url(info.movie_url.as_str()).await {
                        Ok(movie) => {
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

        return movies.into_iter().flatten().collect();
    }
}

#[derive(Debug)]
pub struct PreliminaryMovieInformation {
    movie_url: String,
    rating: Option<f32>,
}
