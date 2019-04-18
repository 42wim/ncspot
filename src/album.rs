use std::fmt;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use rspotify::spotify::model::album::{FullAlbum, SavedAlbum, SimplifiedAlbum};

use library::Library;
use queue::Queue;
use spotify::Spotify;
use track::Track;
use traits::ListItem;

#[derive(Clone, Deserialize, Serialize)]
pub struct Album {
    pub id: String,
    pub title: String,
    pub artists: Vec<String>,
    pub artist_ids: Vec<String>,
    pub year: String,
    pub cover_url: Option<String>,
    pub url: String,
    pub tracks: Option<Vec<Track>>,
    pub added_at: Option<DateTime<Utc>>
}

impl Album {
    pub fn load_tracks(&mut self, spotify: Arc<Spotify>) {
        if self.tracks.is_some() {
            return;
        }

        if let Some(fa) = spotify.full_album(&self.id) {
            self.tracks = Some(
                fa.tracks
                    .items
                    .iter()
                    .map(|st| Track::from_simplified_track(&st, &fa))
                    .collect(),
            );
        }
    }
}

impl From<&SimplifiedAlbum> for Album {
    fn from(sa: &SimplifiedAlbum) -> Self {
        Self {
            id: sa.id.clone(),
            title: sa.name.clone(),
            artists: sa.artists.iter().map(|sa| sa.name.clone()).collect(),
            artist_ids: sa.artists.iter().map(|sa| sa.id.clone()).collect(),
            year: sa.release_date.split('-').next().unwrap().into(),
            cover_url: sa.images.get(0).map(|i| i.url.clone()),
            url: sa.uri.clone(),
            tracks: None,
            added_at: None,
        }
    }
}

impl From<&FullAlbum> for Album {
    fn from(fa: &FullAlbum) -> Self {
        let tracks = Some(
            fa.tracks
                .items
                .iter()
                .map(|st| Track::from_simplified_track(&st, &fa))
                .collect(),
        );

        Self {
            id: fa.id.clone(),
            title: fa.name.clone(),
            artists: fa.artists.iter().map(|sa| sa.name.clone()).collect(),
            artist_ids: fa.artists.iter().map(|sa| sa.id.clone()).collect(),
            year: fa.release_date.split('-').next().unwrap().into(),
            cover_url: fa.images.get(0).map(|i| i.url.clone()),
            url: fa.uri.clone(),
            tracks,
            added_at: None,
        }
    }
}

impl From<&SavedAlbum> for Album {
    fn from(sa: &SavedAlbum) -> Self {
        let mut album: Self = (&sa.album).into();
        album.added_at = Some(sa.added_at);
        album
    }
}

impl fmt::Display for Album {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.artists.join(", "), self.title)
    }
}

impl fmt::Debug for Album {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({} - {} ({}))",
            self.artists.join(", "),
            self.title,
            self.id
        )
    }
}

impl ListItem for Album {
    fn is_playing(&self, queue: Arc<Queue>) -> bool {
        if let Some(tracks) = self.tracks.as_ref() {
            let playing: Vec<String> = queue
                .queue
                .read()
                .unwrap()
                .iter()
                .map(|t| t.id.clone())
                .collect();
            let ids: Vec<String> = tracks.iter().map(|t| t.id.clone()).collect();
            !ids.is_empty() && playing == ids
        } else {
            false
        }
    }

    fn display_left(&self) -> String {
        format!("{}", self)
    }

    fn display_right(&self, library: Arc<Library>) -> String {
        let saved = if library.is_saved_album(self) {
            if library.use_nerdfont {
                "\u{f62b} "
            } else {
                "✓ "
            }
        } else {
            ""
        };
        format!("{}{}", saved, self.year)
    }

    fn play(&mut self, queue: Arc<Queue>) {
        self.load_tracks(queue.get_spotify());

        if let Some(tracks) = self.tracks.as_ref() {
            let tracks: Vec<&Track> = tracks.iter().collect();
            let index = queue.append_next(tracks);
            queue.play(index, true);
        }
    }

    fn queue(&mut self, queue: Arc<Queue>) {
        self.load_tracks(queue.get_spotify());

        if let Some(tracks) = self.tracks.as_ref() {
            for t in tracks {
                queue.append(&t);
            }
        }
    }
}
