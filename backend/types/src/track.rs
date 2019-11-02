/*
 *  YAMBA types
 *  Copyright (C) 2019 Aron Heinecke
 *
 *  This program is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  This program is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

//! Daemon internal structures
//!
//! Defined here to allow direct conversion with public representation and further leverage the complexity in daemon

use metrohash::MetroHash128;
use serde::Deserialize;

use core::hash::{Hash, Hasher};

pub trait GetId {
    /// Calculates ID for Object
    fn get_id(&self) -> String;
}

/// Response of either tracklist of playlist
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TrackResponse {
    TrackList(TrackList),
    Track(Track),
}

/// Internal representation of a "playlist" of tracks
#[derive(Debug, Deserialize)]
pub struct TrackList {
    pub title: String,
    pub id: String,
    /// This should always contain "playlist", not present for tracks
    pub _type: String,
    /// ytdl extractor (internal stuff but useful for ID)
    pub extractor: String,
    pub webpage_url: String,
    pub entries: Vec<Track>,
    pub uploader: Option<String>,
}

impl Hash for TrackList {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.extractor.hash(state);
        self.uploader.hash(state);
    }
}

impl GetId for TrackList {
    fn get_id(&self) -> String {
        let mut hasher = MetroHash128::default();
        self.hash(&mut hasher);
        let (h1, h2) = hasher.finish128();
        format!("{:x}{:x}", h1, h2)
    }
}

/// Internal representation of a Song
#[derive(Debug, Deserialize)]
pub struct Track {
    pub title: String,
    pub id: String,
    /// ytdl extractor (internal stuff but useful for ID)
    pub extractor: String,
    pub duration: Option<f64>,
    pub formats: Vec<Format>,
    pub protocol: Option<String>,
    pub webpage_url: String,
    pub artist: Option<String>,
    pub uploader: Option<String>,
}

/// Track format information
#[derive(Debug, Deserialize)]
pub struct Format {
    pub filesize: Option<i64>,
    pub format: String,
    /// audio bit rate
    pub abr: Option<i64>,
    pub format_id: String,
    pub url: String,
    pub protocol: Option<String>,
    pub vcodec: Option<String>,
    /// audio codec
    pub acodec: Option<String>,
    pub http_headers: HttpHeaders,
}

impl Hash for Track {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.extractor.hash(state);
        self.uploader.hash(state);
    }
}

impl Track {
    /// Takes the artist of this track
    /// Can be uploader/artist
    pub fn take_artist(&mut self) -> Option<String> {
        if let Some(v) = self.artist.take() {
            return Some(v);
        } else if let Some(v) = self.uploader.take() {
            return Some(v);
        }
        None
    }

    /// Duration as u32
    pub fn duration_as_u32(&self) -> Option<u32> {
        if let Some(v) = self.duration {
            Some(v as u32)
        } else {
            None
        }
    }

    /// Returns best audio format
    pub fn best_audio_format(&self, min_audio_bitrate: i64) -> Option<&Format> {
        let track_audio = self.best_audio_only_format();
        let track_mixed = self.best_mixed_audio_format();

        if let Some(audio_track) = track_audio {
            if let Some(mixed_track) = track_mixed {
                let abr_audio = audio_track
                    .abr
                    .expect("No audio bitrate in audio-only track!");
                if abr_audio >= mixed_track.abr.unwrap() || abr_audio >= min_audio_bitrate {
                    return Some(audio_track);
                } else {
                    return Some(mixed_track);
                }
            } else {
                track_mixed
            }
        } else {
            // trace!("Using fallback track..");
            self.formats.get(0)
        }
    }

    /// Returns bests audio format with video
    pub fn best_mixed_audio_format(&self) -> Option<&Format> {
        Track::filter_best_audio_format(self.mixed_only_formats())
    }

    /// Returns format with best audio bitrate from input
    pub fn filter_best_audio_format(formats: Vec<&Format>) -> Option<&Format> {
        let mut max_bitrate: i64 = -1;
        let mut max_format: Option<&Format> = None;

        formats.into_iter().for_each(|format| {
            if let Some(bitrate) = format.abr {
                if max_bitrate < bitrate {
                    max_bitrate = bitrate;
                    max_format = Option::Some(format);
                }
            }
        });
        max_format
    }

    /// Returns best only-audio format
    pub fn best_audio_only_format(&self) -> Option<&Format> {
        Track::filter_best_audio_format(self.audio_only_formats())
    }

    /// Return audio+video formats
    pub fn mixed_only_formats(&self) -> Vec<&Format> {
        self.formats
            .iter()
            .filter(|f| f.has_audio() && f.has_video())
            .collect()
    }

    /// Return audio only formats
    pub fn audio_only_formats(&self) -> Vec<&Format> {
        self.formats.iter().filter(|f| f.is_audio_only()).collect()
    }
}

impl Format {
    pub fn has_audio(&self) -> bool {
        match self.acodec {
            Some(ref ac) => ac != "none",
            None => false,
        }
    }
    pub fn has_video(&self) -> bool {
        match self.vcodec {
            Some(ref vc) => vc != "none",
            None => false,
        }
    }
    pub fn is_audio_only(&self) -> bool {
        !self.has_video() && self.has_audio()
    }
}

/// Http headers, could be required for some URIs to work
///   
/// Currently unused, would have to be passed to gstreamer on playback
#[derive(Debug, Deserialize)]
pub struct HttpHeaders {
    #[serde(rename = "Accept-Charset")]
    pub accept_charset: String,
    #[serde(rename = "Accept-Language")]
    pub accept_language: String,
    #[serde(rename = "Accept-Encoding")]
    pub accept_encoding: String,
    #[serde(rename = "Accept")]
    pub accept: String,
    #[serde(rename = "User-Agent")]
    pub user_agent: String,
}
