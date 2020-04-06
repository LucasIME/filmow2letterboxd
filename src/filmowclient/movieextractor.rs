use select::document::Document;
use select::predicate::Name;

use crate::filmowclient::Movie;

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
}
