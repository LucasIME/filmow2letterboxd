mod movieextractor;
use movieextractor::MovieExtractor;

pub mod movie;
use movie::Movie;

#[derive(Debug)]
pub struct FilmowClient {}

impl FilmowClient {
    pub async fn get_all_movies_from_watchlist(user: &str) -> Vec<Movie> {
        println!("Fetching watchlist for user {}", user);

        let number_of_pages = FilmowClient::get_last_watchlist_page_number(&user).await;
        println!("Number of watchlist pages {:?}", number_of_pages);

        let mut resp = vec![];
        for page_num in 1..=number_of_pages {
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
                }
                _ => break,
            }
        }

        resp
    }

    pub async fn get_all_watched_movies(user: &str) -> Vec<Movie> {
        println!("Fetching watched movies for user {}", user);

        let number_of_pages = FilmowClient::get_last_watched_page_number(&user).await;
        println!("Number of watched movies pages {:?}", number_of_pages);

        let mut resp = vec![];
        for page_num in 1..=number_of_pages {
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
        resp
    }

    async fn get_last_watchlist_page_number(user: &str) -> i32 {
        println!("Getting total number of watchlist pages");
        let watchlist_url = FilmowClient::get_watchlist_url_for_page(user, 1);
        match FilmowClient::get_html_from_url(watchlist_url.as_str()).await {
            Ok(watchlist_page_html) => {
                MovieExtractor::get_last_page_from_html(watchlist_page_html.as_str()).unwrap_or(1)
            }
            Err(e) => {
                panic!("Error when trying to find number of watchlist pages: {}", e);
            }
        }
    }

    async fn get_last_watched_page_number(user: &str) -> i32 {
        println!("Getting total number of watched pages");
        let watched_url = FilmowClient::get_watched_url_for_page(user, 1);
        match FilmowClient::get_html_from_url(watched_url.as_str()).await {
            Ok(watched_page_html) => {
                MovieExtractor::get_last_page_from_html(watched_page_html.as_str()).unwrap_or(1)
            }
            Err(e) => {
                panic!("Error when trying to find number of watched pages: {}", e);
            }
        }
    }
    fn get_base_url() -> String {
        "https://filmow.com".to_string()
    }

    fn get_watchlist_url_for_page(user: &str, page: i32) -> String {
        if page == 1 {
            return format!("https://filmow.com/usuario/{}/quero-ver/", user);
        }

        format!(
            "https://filmow.com/usuario/{}/quero-ver/?pagina={}",
            user, page
        )
    }

    fn get_watched_url_for_page(user: &str, page: i32) -> String {
        if page == 1 {
            return format!("https://filmow.com/usuario/{}/ja-vi/", user);
        }

        format!("https://filmow.com/usuario/{}/ja-vi/?pagina={}", user, page)
    }

    async fn get_html_from_url(url: &str) -> Result<String, String> {
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
            Err(e) => Err(format!(
                "Failed to get HTML for url: {}. Received error: {:?}",
                url, e
            )),
        }
    }

    async fn get_movie_from_url(url: &str) -> Result<Movie, String> {
        match FilmowClient::get_html_from_url(url).await {
            Ok(html_body) => MovieExtractor::extract_movie_from_html(html_body.as_str(), url),
            Err(e) => Err(e),
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

        movies.into_iter().flatten().collect()
    }
}

#[derive(Debug)]
pub struct PreliminaryMovieInformation {
    movie_url: String,
    rating: Option<f32>,
}
