use csv::Writer;

use crate::filmowclient::movie::Movie;

pub struct CsvWriter {}

impl CsvWriter {
    pub fn save_movies_to_csv(movies: Vec<Movie>, file_name: &str) -> Result<(), String> {
        let mut wrt = Writer::from_path(file_name)
            .unwrap_or_else(|_| panic!("Could not create CSV Writer for file {}", file_name));
        if let Err(e) = wrt.write_record(&Movie::csv_titles()) {
            return Err(format!(
                "Error when adding header to Csv file {}. {:?}",
                file_name, e
            ));
        }
        for movie in movies.iter() {
            if let Err(e) = wrt.write_record(movie.to_csvable_array()) {
                return Err(format!(
                    "Error when adding entry to Csv file {}. Entry: {:?}, Error:{:?}",
                    file_name, movie, e
                ));
            }
        }

        if let Err(e) = wrt.flush() {
            return Err(format!("Error when flushing file {}. {:?}", file_name, e));
        }

        Ok(())
    }
}
