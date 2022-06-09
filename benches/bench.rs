#![feature(test)]
#![feature(bench_black_box)]
#![allow(clippy::struct_excessive_bools)]

extern crate test;

use jayson::{DeserializeFromValue, Error, IntoValue};
use serde_derive::{Deserialize, Serialize};
use test::Bencher;

fn input_json() -> String {
    std::fs::read_to_string("benches/twitter.json").unwrap()
}

#[bench]
fn bench_deserialize_jayson(b: &mut Bencher) {
    let j = input_json();
    b.iter(|| {
        let json = serde_json::from_str::<serde_json::Value>(&j).unwrap();
        let _t = Twitter::deserialize_from_value(json.into_value()).unwrap();
    });
}

#[bench]
fn bench_deserialize_serdejson(b: &mut Bencher) {
    let j = input_json();
    b.iter(|| {
        serde_json::from_str::<Twitter>(&j).unwrap();
    });
}
#[bench]
fn bench_deserialize_serdejson_value(b: &mut Bencher) {
    let j = input_json();
    b.iter(|| {
        serde_json::from_str::<serde_json::Value>(&j).unwrap();
    });
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Twitter {
    statuses: Vec<Status>,
    search_metadata: SearchMetadata,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Status {
    metadata: Metadata,
    created_at: String,
    id: u64,
    id_str: String,
    text: String,
    source: String,
    truncated: bool,
    in_reply_to_status_id: Option<u64>,
    in_reply_to_status_id_str: Option<String>,
    in_reply_to_user_id: Option<u32>,
    in_reply_to_user_id_str: Option<String>,
    in_reply_to_screen_name: Option<String>,
    user: User,
    geo: (),
    coordinates: (),
    place: (),
    contributors: (),
    retweeted_status: Option<Box<Status>>,
    retweet_count: u32,
    favorite_count: u32,
    entities: StatusEntities,
    favorited: bool,
    retweeted: bool,
    possibly_sensitive: Option<bool>,
    lang: String,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Metadata {
    result_type: String,
    iso_language_code: String,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct User {
    id: u32,
    id_str: String,
    name: String,
    screen_name: String,
    location: String,
    description: String,
    url: Option<String>,
    entities: UserEntities,
    protected: bool,
    followers_count: u32,
    friends_count: u32,
    listed_count: u32,
    created_at: String,
    favourites_count: u32,
    utc_offset: Option<i32>,
    time_zone: Option<String>,
    geo_enabled: bool,
    verified: bool,
    statuses_count: u32,
    lang: String,
    contributors_enabled: bool,
    is_translator: bool,
    is_translation_enabled: bool,
    profile_background_color: String,
    profile_background_image_url: String,
    profile_background_image_url_https: String,
    profile_background_tile: bool,
    profile_image_url: String,
    profile_image_url_https: String,
    profile_banner_url: Option<String>,
    profile_link_color: String,
    profile_sidebar_border_color: String,
    profile_sidebar_fill_color: String,
    profile_text_color: String,
    profile_use_background_image: bool,
    default_profile: bool,
    default_profile_image: bool,
    following: bool,
    follow_request_sent: bool,
    notifications: bool,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct UserEntities {
    url: Option<UserUrl>,
    description: UserEntitiesDescription,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct UserUrl {
    urls: Vec<Url>,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Url {
    url: String,
    expanded_url: String,
    display_url: String,
    indices: Indices,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct UserEntitiesDescription {
    urls: Vec<Url>,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct StatusEntities {
    hashtags: Vec<Hashtag>,
    symbols: Vec<()>,
    urls: Vec<Url>,
    user_mentions: Vec<UserMention>,
    media: Option<Vec<Media>>,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Hashtag {
    text: String,
    indices: Indices,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct UserMention {
    screen_name: String,
    name: String,
    id: u32,
    id_str: String,
    indices: Indices,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Media {
    id: u64,
    id_str: String,
    indices: Indices,
    media_url: String,
    media_url_https: String,
    url: String,
    display_url: String,
    expanded_url: String,
    #[jayson(rename = "type")]
    #[serde(rename = "type")]
    media_type: String,
    sizes: Sizes,
    source_status_id: Option<u64>,
    source_status_id_str: Option<String>,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Sizes {
    medium: Size,
    small: Size,
    thumb: Size,
    large: Size,
}

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct Size {
    w: u16,
    h: u16,
    resize: String,
}

type Indices = (u8, u8);

#[derive(Serialize, Deserialize, DeserializeFromValue)]
#[jayson(error = Error)]
struct SearchMetadata {
    completed_in: f32,
    max_id: u64,
    max_id_str: String,
    next_results: String,
    query: String,
    refresh_url: String,
    count: u8,
    since_id: u64,
    since_id_str: String,
}
