use url::{Url, ParseError};
use chrono::{DateTime, Utc};
use std::Path;

struct PodcastConfig {
  title: String,
  subtitle: String, //itunes
  xmlUrl: Url,
  siteUrl: Url,
  description: String,
  summary: String, //itunes
  lastBuild: DateTime<Utc>,
  author: String,
  ownerName: String, //itunes
  ownerEmail: String, //itunes
  image: Url, //itunes 1400 x 1400 min, max 3000 x 3000
  category: String, //TODO support multiple categories
  explicit: bool, //itunes
}

struct PodcastEpisode {
  title: String, //id3 title "TIT2"
  episodeUrl: Url, //derived
  author: String, //config
  description: String, //id3 description "TIT3"
  summary: String, //id3 comment "COMM"
  audioUrl: Url, //derived
  guid: Url, //copy the audiourl
  pubDate: DateTime<Utc>, //file metadata date modified
  duration: String, //HH:MM:SS (mp3 duration crate)
  keywords: Vec<String>, //config
  category: String, //config
  explicit: bool, //config
}

}

struct Podcast {
  meta: PodcastConfig
  episodes: Vec<PodcastEpisode>
}

impl PodcastConfig {
  fn from_toml(toml: Path) -> {
    
  }
}
