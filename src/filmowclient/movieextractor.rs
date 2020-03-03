use select::document::Document;
use select::predicate::Name;

use crate::filmowclient::Movie;

#[derive(Debug)]
pub struct MovieExtractor {}

impl MovieExtractor {
    pub fn extract_movie_from_html(html_body: &str) -> Result<Movie, String> {
        let title = MovieExtractor::extract_title(html_body);
        let director = MovieExtractor::extract_director(html_body);
        let year = MovieExtractor::extract_year(html_body);

        return Ok(Movie {
            title: title,
            director: director,
            year: year,
        });
    }

    fn extract_title(resp: &str) -> String {
        return Document::from(resp)
            .find(Name("h2"))
            .filter(|n| {
                n.attr("class").is_some() && n.attr("class").unwrap() == "movie-original-title"
            })
            .map(|n| n.text())
            .nth(0)
            .expect("Could not extract title for movie");
    }

    fn extract_director(resp: &str) -> String {
        return Document::from(resp)
            .find(Name("span"))
            .filter(|n| n.attr("itemprop").is_some() && n.attr("itemprop").unwrap() == "director")
            .map(|n| n.text().trim().to_string())
            .nth(0)
            .expect("Could not extract director for movie");
    }

    fn extract_year(resp: &str) -> u32 {
        return Document::from(resp)
            .find(Name("small"))
            .filter(|n| n.attr("class").is_some() && n.attr("class").unwrap() == "release")
            .map(|n| n.text())
            .nth(0)
            .expect("Could not find year string for movie")
            .parse::<u32>()
            .expect("Could not parse year string into u32");
    }
}
