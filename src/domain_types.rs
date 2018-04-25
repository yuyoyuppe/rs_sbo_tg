#![allow(dead_code)]

use chrono::prelude::{DateTime, Utc};
use std::time::Duration;

pub struct User {
  feeds: Vec<u32>,
  categories: Vec<u32>,
  pub name: String,
  pub whitelisted_words: Vec<String>,
  pub cooldown: Duration
}

pub struct Feed {
  pub url: String
}

pub struct FeedData {
  user_id: u32,
  feed_id: u32,
  category_id: u32,
  pub last_item_sent: DateTime<Utc>,
  pub cooldown: Duration
}

pub struct Category {
  user_id: u32,
  category_id: u32,
  pub whitelisted_words: Vec<String>,
  pub name: String,
  pub cooldown: Duration
}
