use super::play::Songs;
use anyhow::Result;
use async_mpd::{Filter, FilterExpr, MpdClient, Tag, Track};

#[derive(Debug, Default)]
pub struct Search {
    current: String,
    results: Songs,
}

impl Search {
    pub async fn search(&mut self, client: &mut MpdClient) -> Result<()> {
        if !self.current.is_empty() {
            // get all songs with provided title, can be expanded to more stuff I am sure
            let filter = Filter::new().and(FilterExpr::Contains(Tag::Title, self.current.clone()));

            // set songs to result of search
            self.results.set_songs(&client.search(&filter).await?);
        } else {
            self.results.set_songs(&[]);
        }

        Ok(())
    }

    pub fn results(&self) -> &[Track] {
        &self.results
    }

    pub fn push(&mut self, c: char) {
        self.current.push(c);
    }

    pub fn pop(&mut self) {
        self.current.pop();
    }

    pub fn clear(&mut self) {
        self.current.clear();
    }

    pub fn get(&self, length: usize) -> &str {
        // accounting for pipe characters at beginning and end, and cursor
        let length = length.checked_sub(3).unwrap_or(length);

        let start = if self.current.len() > length {
            self.current.len() - length
        } else {
            0
        };
        &self.current[start..]
    }
}
