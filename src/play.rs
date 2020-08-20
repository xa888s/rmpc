use async_mpd::{Status, Track};
use std::ops::{Deref, DerefMut, Index, IndexMut};

#[derive(Debug, Default)]
pub struct Songs {
    songs: Vec<Track>,
    status: Option<Status>,
}

impl Songs {
    pub fn status(&self) -> Option<&Status> {
        self.status.as_ref()
    }

    pub fn set_status(&mut self, status: Option<Status>) {
        self.status = status;
    }

    pub fn set_songs(&mut self, songs: Vec<Track>) {
        self.songs = songs;
    }
}

impl Deref for Songs {
    type Target = Vec<Track>;
    fn deref(&self) -> &Self::Target {
        &self.songs
    }
}

impl DerefMut for Songs {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.songs
    }
}

impl Index<usize> for Songs {
    type Output = Track;
    fn index(&self, index: usize) -> &Self::Output {
        &self.songs[index]
    }
}

impl IndexMut<usize> for Songs {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.songs[index]
    }
}
