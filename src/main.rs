use reqwest;
use select::document::Document;
use select::predicate::Name;


#[derive(Debug)]
struct FilmowClient {
    user: &'static str,
}

impl FilmowClient {
    fn new(user: &'static str) -> FilmowClient {
        FilmowClient { user }
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
}

fn has_attr_with_name(node: &select::node::Node, name: &str) -> bool {
    node.attr(name).is_some()
}

fn main() {
    let user = "lucasmeireles33";
    let client = FilmowClient::new(user);
    let url = client.get_watchlist_url();
    client.iterate_through_pages();

    let resp = reqwest::get(url.as_str()).unwrap();
    Document::from_read(resp).unwrap()
        .find(Name("a"))
        .filter(|n| has_attr_with_name(n, "data-movie-pk"))
        .for_each(|x|println!("{:?}", x));
}
