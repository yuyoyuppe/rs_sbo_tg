#![allow(dead_code, unused_variables, unused_macros)]
#![feature(concat_idents)]

extern crate chrono;
extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

mod domain_types;
pub mod rss_bot;

use rss_bot::RssBot;
use std::env;

fn main() {
  let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
  let reactor = tokio_core::reactor::Core::new().unwrap();
  RssBot::new(token, reactor.handle()).run(reactor);
}
