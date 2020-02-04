use csv::Writer;
use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::env;
use std::thread;

#[derive(Debug)]
struct FilmowClient {}

#[derive(Debug)]
struct Movie {
    title: String,
    director: String,
    year: u32,
}

impl Movie {
    fn to_csvable_array(&self) -> Vec<String> {
        return vec![
            self.title.clone(),
            self.director.clone(),
            self.year.to_string(),
        ];
    }
}

impl FilmowClient {
    fn new() -> FilmowClient {
        FilmowClient {}
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

    fn get_movie_links_from_url(&self, url: &str) -> Result<Vec<String>, &str> {
        println!("Fetching links from Page {}", url);
        match reqwest::get(url) {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found");
                }

                Ok(Document::from_read(resp)
                    .unwrap()
                    .find(Name("a"))
                    .filter(|n| has_attr_with_name(n, "data-movie-pk"))
                    .map(|n| n.attr("href"))
                    .flatten()
                    .map(|x| self.get_base_url() + &x.to_string())
                    .collect())
            }
            _ => {
                return Err("Non Ok");
            }
        }
    }

    fn extract_title(&self, resp: &str) -> String {
        return Document::from(resp)
            .find(Name("h2"))
            .filter(|n| {
                n.attr("class").is_some() && n.attr("class").unwrap() == "movie-original-title"
            })
            .map(|n| n.text())
            .nth(0)
            .unwrap();
    }

    fn extract_director(&self, resp: &str) -> String {
        return Document::from(resp)
            .find(Name("span"))
            .filter(|n| n.attr("itemprop").is_some() && n.attr("itemprop").unwrap() == "director")
            .map(|n| n.text().trim().to_string())
            .nth(0)
            .unwrap();
    }

    fn extract_year(&self, resp: &str) -> u32 {
        return Document::from(resp)
            .find(Name("small"))
            .filter(|n| n.attr("class").is_some() && n.attr("class").unwrap() == "release")
            .map(|n| n.text())
            .nth(0)
            .unwrap()
            .parse::<u32>()
            .unwrap();
    }

    fn get_movie_from_url(&self, url: &str) -> Result<Movie, &str> {
        match reqwest::get(url) {
            Ok(mut resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found");
                }

                let html_body = match resp.text() {
                    Ok(body) => body,
                    _ => {
                        return Err("Error when getting html body");
                    }
                };

                let title = self.extract_title(html_body.as_str());
                let director = self.extract_director(html_body.as_str());
                let year = self.extract_year(html_body.as_str());

                return Ok(Movie {
                    title: title,
                    director: director,
                    year: year,
                });
            }
            _ => {
                return Err("Non Ok");
            }
        }
    }

    fn get_all_movies_from_watchlist(&self, user: &str) -> Vec<Movie> {
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            match self
                .get_movie_links_from_url(self.get_watchlist_url_for_page(user, page_num).as_str())
            {
                Ok(links) => {
                    let mut page_movies = parallel_process_links(links);
                    println!("Movies for page {}: {:?}", page_num, page_movies);
                    resp.append(&mut page_movies);
                    page_num += 1;
                }
                _ => break,
            }
        }

        return resp;
    }

    fn get_all_watched_movies(&self, user: &str) -> Vec<Movie> {
        let mut resp = vec![];
        let mut page_num = 1;
        loop {
            match self
                .get_movie_links_from_url(self.get_watched_url_for_page(user, page_num).as_str())
            {
                Ok(links) => {
                    let mut page_movies = parallel_process_links(links);
                    println!("Movies for page {}: {:?}", page_num, page_movies);
                    resp.append(&mut page_movies);
                    page_num += 1;
                }
                _ => break,
            }
        }
        return resp;
    }
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}

fn parallel_process_links(links: Vec<String>) -> Vec<Movie> {
    let mut children = vec![];
    for link in links {
        children.push(thread::spawn(move || -> Movie {
            FilmowClient::new().get_movie_from_url(&link).unwrap()
        }));
    }

    let mut movies = vec![];
    for child in children {
        let movie = child.join().unwrap();
        movies.push(movie);
    }

    return movies;
}

fn save_movies_to_csv(movies: Vec<Movie>, file_name: &str) {
    let mut wrt = Writer::from_path(file_name).unwrap();
    wrt.write_record(&["Title", "Directors", "Year"]);
    for movie in movies.iter() {
        wrt.write_record(movie.to_csvable_array());
    }
    wrt.flush();
}

fn main() {
    let user = env::args().nth(1).unwrap();
    let client = FilmowClient::new();
    let watchlist_movies = client.get_all_movies_from_watchlist(user.as_str());
    save_movies_to_csv(watchlist_movies, "watchlist.csv");
    let watched_movies = client.get_all_watched_movies(user.as_str());
    save_movies_to_csv(watched_movies, "watched.csv");
}
