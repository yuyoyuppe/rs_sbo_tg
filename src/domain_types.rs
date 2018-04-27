extern crate telegram_bot;

use chrono::prelude::{DateTime, Utc};
use std::time::Duration;

#[derive(Debug)]
pub struct User {
  pub feeds:             Vec<Feed>,
  pub categories:        Vec<u32>,
  pub telegram_user_id:  telegram_bot::UserId,
  pub whitelisted_words: Vec<String>,
  pub cooldown:          Duration
}

impl Default for User {
  fn default() -> User {
    User {
      telegram_user_id:  telegram_bot::UserId::from(0),
      feeds:             Default::default(),
      categories:        Default::default(),
      whitelisted_words: Default::default(),
      cooldown:          Default::default()
    }
  }
}

#[derive(Debug)]
pub struct Feed {
  pub url: String
}

#[derive(Debug)]
pub struct FeedData {
  user_id:            u32,
  feed_id:            u32,
  category_id:        u32,
  pub last_item_sent: DateTime<Utc>,
  pub cooldown:       Duration
}

pub struct Category {
  user_id:               u32,
  category_id:           u32,
  pub whitelisted_words: Vec<String>,
  pub name:              String,
  pub cooldown:          Duration
}
