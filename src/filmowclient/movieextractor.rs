use select::document::Document;
use select::predicate::And;
use select::predicate::Class;
use select::predicate::Name;

use crate::filmowclient::FilmowClient;
use crate::filmowclient::Movie;
use crate::filmowclient::PreliminaryMovieInformation;

#[derive(Debug)]
pub struct MovieExtractor {}

impl MovieExtractor {
    pub fn extract_movie_from_html(html_body: &str, url: &str) -> Result<Movie, String> {
        let title = MovieExtractor::extract_title(html_body);
        let director = MovieExtractor::extract_director(html_body);
        let year = MovieExtractor::extract_year(html_body);

        if title.is_none() {
            return Err(format!("Could not extract title from page: {}", url));
        }

        if director.is_none() {
            return Err(format!("Could not extract director from page: {}", url));
        }

        if year.is_none() {
            return Err(format!("Could not extract year from page: {}", url));
        }

        return Ok(Movie {
            title: title.unwrap(),
            director: director.unwrap(),
            year: year.unwrap(),
            rating: None,
        });
    }

    fn extract_title(resp: &str) -> Option<String> {
        return Document::from(resp)
            .find(Name("h2"))
            .filter(|n| {
                n.attr("class").is_some() && n.attr("class").unwrap() == "movie-original-title"
            })
            .map(|n| n.text())
            .nth(0);
    }

    fn extract_director(resp: &str) -> Option<String> {
        return Document::from(resp)
            .find(Name("span"))
            .filter(|n| n.attr("itemprop").is_some() && n.attr("itemprop").unwrap() == "director")
            .map(|n| n.text().trim().to_string())
            .nth(0);
    }

    fn extract_year(resp: &str) -> Option<u32> {
        return Document::from(resp)
            .find(Name("small"))
            .filter(|n| n.attr("class").is_some() && n.attr("class").unwrap() == "release")
            .map(|n| n.text())
            .nth(0)
            .map_or(None, |s| match s.parse::<u32>() {
                Ok(num) => Some(num),
                Err(_e) => None,
            });
    }

    pub fn get_preliminary_info_for_watchlist(
        watchlist_page_html: &str,
    ) -> Vec<PreliminaryMovieInformation> {
        return Document::from(watchlist_page_html)
            .find(Name("a"))
            .filter(|n| n.attr("data-movie-pk").is_some())
            .map(|n| n.attr("href"))
            .flatten()
            .map(|x| FilmowClient::get_base_url() + &x.to_string())
            .map(|url| PreliminaryMovieInformation {
                movie_url: url,
                rating: None,
            })
            .collect();
    }

    pub fn get_preliminary_info_for_watched_movies(
        watched_page_html: &str,
    ) -> Vec<PreliminaryMovieInformation> {
        let html_per_movie = MovieExtractor::break_watched_movies_html_per_movie(watched_page_html);

        match html_per_movie {
            Ok(html_vec) => {
                return html_vec
                    .iter()
                    .map(|movie_html| MovieExtractor::extract_watched_movie_info(movie_html))
                    .flatten()
                    .collect();
            }
            _ => vec![],
        }
    }

    pub fn get_last_page_from_html(page_html: &str) -> Option<i32> {
        let document = Document::from(page_html);

        let last_page = document
            .find(Name("a"))
            .flat_map(|n| n.attr("href"))
            .flat_map(|link| {
                let page_num_str = link.split("pagina=").nth(1);
                return page_num_str.map(|num_str| num_str.parse::<i32>().unwrap());
            })
            .last();

        return last_page;
    }

    fn extract_watched_movie_info(watched_movie_html: &str) -> Option<PreliminaryMovieInformation> {
        let document = Document::from(watched_movie_html);
        let url = document.find(Name("a")).map(|n| n.attr("href")).nth(0);

        let rating: Option<f32> = Document::from(watched_movie_html)
            .find(And(Name("span"), Class("stars")))
            .flat_map(|n| n.attr("title"))
            .flat_map(|s| s.split_ascii_whitespace().nth(1))
            .flat_map(|s| s.parse::<f32>())
            .nth(0);

        return Some(PreliminaryMovieInformation {
            movie_url: FilmowClient::get_base_url() + url??,
            rating: rating,
        });
    }

    fn break_watched_movies_html_per_movie(
        full_watched_page_html: &str,
    ) -> Result<Vec<String>, String> {
        return Ok(Document::from(full_watched_page_html)
            .find(And(Name("li"), Class("movie_list_item")))
            .filter(|n| n.attr("data-movie-pk").is_some())
            .map(|n| n.html())
            .collect());
    }
}
