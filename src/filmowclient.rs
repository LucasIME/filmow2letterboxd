use reqwest;
use select::document::Document;
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

    pub async fn get_all_movies_from_watchlist(user_c: &str) -> Vec<Movie> {
        let user = user_c.to_string().clone();
        println!("Fetching watchlist for user {}", user);
        let mut page_num = 1;
        let total_pages = FilmowClient::get_number_of_watchlist_pages(user_c).await;
        println!("Total number of watchlist pages: {}", total_pages);

        let mut links = vec![];
        let mut children = vec![];
        while page_num <= total_pages {
            let cur_page = page_num;
            let user_clone = user.clone();
            children.push(tokio::spawn(async move {
                let url = FilmowClient::get_watchlist_url_for_page(&user_clone, cur_page);
                FilmowClient::get_movie_links_from_url(&url).await
            }));
            page_num += 1;
        }

        for child in children.iter_mut() {
            let stored_resp = match child.await {
                Ok(v) => v,
                _ => panic!("failed joining tokio!!"),
            };

            match stored_resp {
                Ok(mut link_v) => links.append(&mut link_v),
                Err(e) => println!("Error with response! {}", e),
            }
        }

        let valid_links = links;

        return FilmowClient::parallel_process_links(valid_links).await;
    }

    pub async fn get_all_watched_movies(user_c: &str) -> Vec<Movie> {
        let user = user_c.to_string().clone();
        println!("Fetching watched movies for user {}", user);
        let mut page_num = 1;
        let total_pages = FilmowClient::get_number_of_watched_pages(user_c).await;
        println!("Total number of watched pages: {}", total_pages);

        let mut links = vec![];
        let mut children = vec![];
        while page_num <= total_pages {
            let cur_page = page_num;
            let user_clone = user.clone();
            children.push(tokio::spawn(async move {
                let url = FilmowClient::get_watched_url_for_page(&user_clone, cur_page);
                FilmowClient::get_movie_links_from_url(&url).await
            }));
            page_num += 1;
        }

        for child in children.iter_mut() {
            let stored_resp = match child.await {
                Ok(v) => v,
                _ => panic!("failed joining tokio!!"),
            };

            match stored_resp {
                Ok(mut link_v) => links.append(&mut link_v),
                Err(e) => println!("Error with response! {}", e),
            }
        }

        let valid_links = links;

        return FilmowClient::parallel_process_links(valid_links).await;
    }

    async fn get_number_of_watchlist_pages(user: &str) -> i32 {
        let mut page_num = 1;
        loop {
            match FilmowClient::get_movie_links_from_url(
                FilmowClient::get_watchlist_url_for_page(user, page_num).as_str(),
            )
            .await
            {
                Ok(links) => {
                    page_num += 1;
                }
                _ => {
                    page_num -= 1;
                    break;
                }
            }
        }
        return page_num;
    }

    async fn get_number_of_watched_pages(user: &str) -> i32 {
        let mut page_num = 1;
        loop {
            match FilmowClient::get_movie_links_from_url(
                FilmowClient::get_watched_url_for_page(user, page_num).as_str(),
            )
            .await
            {
                Ok(links) => {
                    page_num += 1;
                }
                _ => {
                    page_num -= 1;
                    break;
                }
            }
        }
        return page_num;
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

    async fn get_movie_links_from_url(url: &str) -> Result<Vec<String>, String> {
        println!("Fetching links from Page {}", url);
        match reqwest::get(url).await {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found".to_string());
                }

                match resp.text().await {
                    Ok(text) => Ok(Document::from(text.as_str())
                        .find(Name("a"))
                        .filter(|n| has_attr_with_name(n, "data-movie-pk"))
                        .map(|n| n.attr("href"))
                        .flatten()
                        .map(|x| FilmowClient::get_base_url() + &x.to_string())
                        .collect()),
                    Err(e) => Err(format!(
                        "Failed to get text from url, {}. Error was: {}",
                        url, e
                    )),
                }
            }
            _ => {
                return Err("Non Ok".to_string());
            }
        }
    }

    async fn get_movie_from_url(url: &str) -> Result<Movie, String> {
        match reqwest::get(url).await {
            Ok(mut resp) => {
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

    async fn parallel_process_links(links: Vec<String>) -> Vec<Movie> {
        let mut children = vec![];
        for link in links {
            children.push(
                tokio::spawn(async move {
                match FilmowClient::get_movie_from_url(&link).await {
                        Ok(movie) => Some(movie),
                        Err(e) => {
                            println!("Could not construct movie from url {}. Ignoring it and continuing. Error was: {}", link, e);
                            None
                        }
                    }
                })
            );
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
pub struct Movie {
    title: String,
    director: String,
    year: u32,
}

impl Movie {
    pub fn to_csvable_array(&self) -> Vec<String> {
        return vec![
            self.title.clone(),
            self.director.clone(),
            self.year.to_string(),
        ];
    }
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}
