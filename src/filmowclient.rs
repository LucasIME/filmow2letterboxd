use reqwest;
use select::document::Document;
use select::predicate::And;
use select::predicate::Class;
use select::predicate::Name;
use std::thread;

mod movieextractor;
use movieextractor::MovieExtractor;

#[derive(Debug)]
pub struct FilmowClient {}

impl FilmowClient {
    pub fn new() -> FilmowClient {
        FilmowClient {}
    }

    pub fn get_all_movies_from_watchlist(&self, user: &str) -> Vec<Movie> {
        println!("Fetching watchlist for user {}", user);
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            match self
                .get_movie_links_from_url(self.get_watchlist_url_for_page(user, page_num).as_str())
            {
                Ok(links) => {
                    let mut page_movies = self.parallel_process_links(links);
                    println!("Movies for page {}: {:?}", page_num, page_movies);
                    resp.append(&mut page_movies);
                    page_num += 1;
                }
                _ => break,
            }
        }

        return resp;
    }

    pub fn get_all_watched_movies(&self, user: &str) -> Vec<Movie> {
        println!("Fetching watched movies for user {}", user);
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            let watched_url_for_page = self.get_watched_url_for_page(user, page_num);
            let html_for_watched_movies =
                FilmowClient::get_html_from_url(watched_url_for_page.as_str());
            match html_for_watched_movies {
                Ok(html_for_watched_page) => {
                    let movie_html_vec = FilmowClient::break_watched_movies_html_per_movie(
                        html_for_watched_page.as_str(),
                    );
                    let movie_info_vec: Vec<WatchedMovieInformation> = movie_html_vec
                        .unwrap()
                        .iter()
                        .flat_map(|movie| MovieExtractor::extract_watched_movie_info(movie))
                        .collect();

                    let movie_info_vec_with_full_url: Vec<WatchedMovieInformation> = movie_info_vec
                        .into_iter()
                        .map(|info| WatchedMovieInformation {
                            movie_url: self.get_base_url() + info.movie_url.as_str(),
                            rating: info.rating,
                        })
                        .collect();

                    let mut page_movies =
                        self.parallel_process_watched_links(movie_info_vec_with_full_url);
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

    fn get_base_url(&self) -> String {
        "https://filmow.com".to_string()
    }

    fn get_watchlist_url_for_page(&self, user: &str, page: i32) -> String {
        format!(
            "https://filmow.com/usuario/{}/filmes/quero-ver/?pagina={}",
            user, page
        )
    }

    fn get_watched_url_for_page(&self, user: &str, page: i32) -> String {
        format!(
            "https://filmow.com/usuario/{}/filmes/ja-vi/?pagina={}",
            user, page
        )
    }

    fn break_watched_movies_html_per_movie(
        full_watched_page_html: &str,
    ) -> Result<Vec<String>, String> {
        return Ok(Document::from(full_watched_page_html)
            .find(And(Name("li"), Class("movie_list_item")))
            .filter(|n| has_attr_with_name(n, "data-movie-pk"))
            .map(|n| n.html())
            .collect());
    }

    fn get_movie_links_from_url(&self, url: &str) -> Result<Vec<String>, String> {
        println!("Fetching links from Page {}", url);
        match FilmowClient::get_html_from_url(url) {
            Ok(html) => {
                return Ok(Document::from(html.as_str())
                    .find(Name("a"))
                    .filter(|n| has_attr_with_name(n, "data-movie-pk"))
                    .map(|n| n.attr("href"))
                    .flatten()
                    .map(|x| self.get_base_url() + &x.to_string())
                    .collect());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn get_html_from_url(url: &str) -> Result<String, String> {
        println!("Getting HTML for url: {}", url);
        match reqwest::get(url) {
            Ok(mut resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found".to_string());
                }

                return Ok(resp.text().unwrap());
            }
            Err(e) => {
                return Err(format!(
                    "Failed to get HTML for url: {}. Reived error: {:?}",
                    url, e
                ));
            }
        }
    }

    fn get_movie_from_url(&self, url: &str) -> Result<Movie, String> {
        match reqwest::get(url) {
            Ok(mut resp) => {
                if resp.status() == 404 {
                    return Err(format!("404 page not found, when fetching for url {}", url));
                }

                let html_body = match resp.text() {
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

    fn parallel_process_watched_links(&self, info_vec: Vec<WatchedMovieInformation>) -> Vec<Movie> {
        let mut children = vec![];
        for info in info_vec {
            children.push(thread::spawn(move || -> Option<Movie> {
                match FilmowClient::new()
                    .get_movie_from_url(info.movie_url.as_str()) {
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
            let movie = child.join().expect("Could not join child thread");
            movies.push(movie);
        }

        return movies.into_iter().flatten().collect();
    }

    fn parallel_process_links(&self, links: Vec<String>) -> Vec<Movie> {
        let mut children = vec![];
        for link in links {
            children.push(thread::spawn(move || -> Option<Movie> {
                match FilmowClient::new()
                    .get_movie_from_url(&link) {
                        Ok(movie) => Some(movie),
                        Err(e) => {
                            println!("Could not construct movie from url {}. Ignoring it and continuing. Error was: {}", link, e);
                            return None;
                        }
                    }
            }));
        }

        let mut movies = vec![];
        for child in children {
            let movie = child.join().expect("Could not join child thread");
            movies.push(movie);
        }

        return movies.into_iter().flatten().collect();
    }
}

#[derive(Debug)]
pub struct Movie {
    title: String,
    director: String,
    year: u32,
    rating: Option<f32>,
}

impl Movie {
    pub fn to_csvable_array(&self) -> Vec<String> {
        return vec![
            self.title.clone(),
            self.director.clone(),
            self.year.to_string(),
            self.rating.map(|r| r.to_string()).unwrap_or("".to_string()),
        ];
    }

    pub fn csv_titles() -> Vec<&'static str> {
        return vec!["Title", "Directors", "Year", "Rating"];
    }
}

#[derive(Debug)]
pub struct WatchedMovieInformation {
    movie_url: String,
    rating: Option<f32>,
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}
