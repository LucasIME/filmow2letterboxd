#[derive(Debug, PartialOrd, PartialEq)]
pub struct Movie {
    pub title: String,
    pub director: String,
    pub year: u32,
    pub rating: Option<f32>,
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
