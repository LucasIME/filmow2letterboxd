use csv::Writer;
use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::thread;

#[derive(Debug)]
struct FilmowClient {
    user: &'static str,
}

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
    fn new(user: &'static str) -> FilmowClient {
        FilmowClient { user }
    }

    fn get_base_url(&self) -> String {
        "https://filmow.com".to_string()
    }

    fn get_watchlist_url_for_page(&self, page: i32) -> String {
        format!(
            "https://filmow.com/usuario/{}/filmes/quero-ver/?pagina={}",
            self.user, page
        )
    }

    fn get_movie_links_from_page(&self, page_num: i32) -> Result<Vec<String>, &str> {
        println!("Fetching links from User {} Page {}", self.user, page_num);
        match reqwest::get(self.get_watchlist_url_for_page(page_num).as_str()) {
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

    fn get_title(&self, resp: &str) -> String {
        return Document::from(resp)
            .find(Name("h2"))
            .filter(|n| {
                n.attr("class").is_some() && n.attr("class").unwrap() == "movie-original-title"
            })
            .map(|n| n.text())
            .nth(0)
            .unwrap();
    }

    fn get_director(&self, resp: &str) -> String {
        return Document::from(resp)
            .find(Name("span"))
            .filter(|n| n.attr("itemprop").is_some() && n.attr("itemprop").unwrap() == "director")
            .map(|n| n.text().trim().to_string())
            .nth(0)
            .unwrap();
    }

    fn get_year(&self, resp: &str) -> u32 {
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

                let title = self.get_title(html_body.as_str());

                let director = self.get_director(html_body.as_str());

                let year = self.get_year(html_body.as_str());

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
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}

fn parallel_process_links(links: Vec<String>) -> Vec<Movie> {
    let mut children = vec![];
    for link in links {
        children.push(thread::spawn(move || -> Movie {
            FilmowClient::new("any_user")
                .get_movie_from_url(&link)
                .unwrap()
        }));
    }

    let mut movies = vec![];
    for child in children {
        let movie = child.join().unwrap();
        movies.push(movie);
    }

    return movies;
}

fn get_all_movies_from_watchlist(client: &FilmowClient) -> Vec<Movie> {
    let mut resp = vec!();
    let mut page_num = 1;
    loop {
        match client.get_movie_links_from_page(page_num) {
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

fn save_movies_to_csv(movies: Vec<Movie>) {
    let mut wrt = Writer::from_path(".csv").unwrap();
    wrt.write_record(&["Title", "Directors", "Year"]);
    for movie in movies.iter() {
        wrt.write_record(movie.to_csvable_array());
    }
    wrt.flush();
}

fn main() {
    let user = "lucasmeireles33";
    let client = FilmowClient::new(user);
    let movies = get_all_movies_from_watchlist(&client);
    println!("{:?}", movies);
    save_movies_to_csv(movies);
}
