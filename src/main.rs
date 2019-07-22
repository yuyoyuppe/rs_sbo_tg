#![allow(unknown_lints, dead_code, unused_variables, unused_macros)]
#![feature(concat_idents, bind_by_move_pattern_guards)]

mod domain_types;
mod feed;
mod helpers;
pub mod rss_bot;

use rss_bot::RssBot;

use failure::{err_msg, Error};

mod tests {
  use super::*;
  use crate::feed::*;
  use std::path::Path;
  static FEEDS_FILENAME: &str = "target\\feeds.txt";

  #[test]
  #[ignore]
  fn feed_downloading_test() -> Result<(), Error> {
    let p = Path::new(FEEDS_FILENAME).canonicalize()?;
    download_test_feeds(&p)?;
    Ok(())
  }

  #[test]
  fn feed_parsing_test() -> Result<(), Error> {
    let p = Path::new(FEEDS_FILENAME).canonicalize()?;
    test_feeds(&p.parent().ok_or(err_msg("couldn't get parent path"))?.join("feeds"))
  }

}

fn main() -> Result<(), Error> {
  let token = std::env::var("TELEGRAM_BOT_TOKEN")?;
  let reactor = tokio_core::reactor::Core::new()?;
  RssBot::new(token, reactor.handle())?.run(reactor)
}
