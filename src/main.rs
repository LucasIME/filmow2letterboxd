use reqwest;
use select::document::Document;
use select::predicate::Name;
use std::thread;

#[derive(Debug)]
struct FilmowClient {
    user: &'static str,
}

impl FilmowClient {

    fn new(user: &'static str) -> FilmowClient {
        FilmowClient { user }
    }

    fn get_base_url(&self) -> String {
        "https://filmow.com".to_string()
    }

    fn get_watchlist_url(&self) -> String {
        format!("https://filmow.com/usuario/{}/filmes/quero-ver", self.user)
    }

    fn get_watchlist_url_for_page(&self, page: i32) -> String {
        format!("https://filmow.com/usuario/{}/filmes/quero-ver/?pagina={}", self.user, page)
    }

    fn iterate_through_pages(&self) {
        let mut page_num = 0;
        loop {
            match reqwest::get(self.get_watchlist_url_for_page(page_num).as_str()) {
                Ok(i) => {
                    if i.status() == 404 {
                        break;
                    }
                    println!("Page number {}", page_num);
                    println!("Page {:?}", i);
                    page_num = page_num + 1;
                },
                _ => { break; }
            }
        }
    }

    fn get_movie_links_from_page(&self, page_num: i32) -> Result<Vec<String>, &str> {
        println!("Fetching links from User {} Page {}", self.user, page_num);
        match reqwest::get(self.get_watchlist_url_for_page(page_num).as_str()) {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found");
                }

                Ok(Document::from_read(resp).unwrap()
                    .find(Name("a"))
                    .filter(|n|has_attr_with_name(n, "data-movie-pk"))
                    .map(|n|n.attr("href"))
                    .flatten()
                    .map(|x|self.get_base_url() + &x.to_string())
                    .collect())
            },
            _ => { return Err("Non Ok") ;}
        }       
    }

    fn get_movie_original_title(&self, url: &str) -> Result<String, &str>{
        println!("Getting original title for url: {}", url);
        match reqwest::get(url) {
            Ok(resp) => {
                if resp.status() == 404 {
                    return Err("404 page not found");
                }

                Ok(Document::from_read(resp).unwrap()
                    .find(Name("h2"))
                    .filter(|n|n.attr("class").is_some() && n.attr("class").unwrap() == "movie-original-title")
                    .map(|n|n.text())
                    .nth(0).unwrap())
                
            },
            _ => { return Err("Non Ok") ;}
        }
    }
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}

fn parallel_process_links(links: Vec<String>) -> Vec<String> {
    let mut children = vec![];
    for link in links {
        children.push(thread::spawn(move || -> String {
            FilmowClient::new("any_user").get_movie_original_title(&link)
            .unwrap_or("failed link ".to_string() + &link)
        }));
    }

    let mut intermediate_names = vec![];
    for child in children {
        let name = child.join().unwrap();
        intermediate_names.push(name.to_string());
    }

    return intermediate_names;
}

fn get_all_movies_from_watchlist(client: &FilmowClient) -> Vec<String> {
    let mut resp = vec![];
    let mut page_num = 0;
    loop {
        match client.get_movie_links_from_page(page_num) { 
            Ok(links) => {
                let mut page_movies = parallel_process_links(links);
                println!("Links for page {}: {:?}", page_num, page_movies);
                resp.append(&mut page_movies);
                page_num += 1;
            },
            _ => break
        }
    }

    return resp;
}


fn main() {
    let user = "lucasmeireles33";
    let client = FilmowClient::new(user);

    println!("{:?}", get_all_movies_from_watchlist(&client));
}
