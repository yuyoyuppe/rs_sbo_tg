#![allow(dead_code, unused_variables, unused_macros)]
#![feature(concat_idents)]

extern crate chrono;
extern crate curl;
extern crate futures;
extern crate quick_xml;
extern crate telegram_bot;
extern crate tokio_core;
extern crate url;
mod domain_types;
pub mod rss_bot;

use rss_bot::RssBot;
use std::env;

use curl::easy::Easy;
use std::{fs::File,
          io::{prelude::*, BufReader, Write}};

fn get_home() -> std::path::PathBuf { env::home_dir().unwrap() }

fn download_test_feeds() {
  let home = get_home();
  let mut feeds_file = home.clone();
  feeds_file.push("feeds.txt");
  let feed_urls = File::open(feeds_file).unwrap();
  let feed_urls = BufReader::new(feed_urls);
  for (n, url) in feed_urls.lines().enumerate() {
    let mut output_filename = home.clone();
    output_filename.push("feeds");
    output_filename.push(n.to_string() + "feed.xml");
    let url = url.unwrap();
    let mut easy = Easy::new();
    easy.follow_location(true).unwrap();
    println!("processing url: {}", url);
    easy.url(&url).unwrap();
    let mut bytes: Vec<u8> = Vec::new();
    {
      let mut transfer = easy.transfer();
      transfer
        .write_function(|data| {
          bytes.extend_from_slice(data);
          Ok(data.len())
        })
        .unwrap();
      transfer.perform().unwrap();
    }
    File::create(&output_filename).unwrap().write_all(&bytes).unwrap();
  }
}

use quick_xml::{events::Event, Reader};

fn parse_some_xml() {
  let mut p = get_home();
  p.push("feeds");
  // for path in std::fs::read_dir(p).unwrap() {
  //   println!("feed: {:#?}", path);
  // }
  let feed0 = std::fs::read_dir(p).unwrap().next().unwrap().unwrap();

  let mut reader = Reader::from_file(feed0.path().to_str().unwrap()).unwrap();
  reader.trim_text(true);

  let mut count = 0;
  let mut txt = Vec::new();
  let mut buf = Vec::new();

  // The `Reader` does not implement `Iterator` because it outputs borrowed data
  // (`Cow`s)
  loop {
    match reader.read_event(&mut buf) {
      // for triggering namespaced events, use this instead:
      // match reader.read_namespaced_event(&mut buf) {
      Ok(Event::Start(ref e)) => {
        // for namespaced:
        // Ok((ref namespace_value, Event::Start(ref e)))
        match e.name() {
          b"tag1" => println!(
            "attributes values: {:?}",
            e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>()
          ),
          b"tag2" => count += 1,
          _ => ()
        }
      }
      // unescape and decode the text event using the reader encoding
      Ok(Event::Text(e)) => txt.push(e.unescape_and_decode(&reader).unwrap()),
      Ok(Event::Eof) => break, // exits the loop when reaching end of file
      Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
      _ => () // There are several other `Event`s we do not consider here
    }

    // if we don't keep a borrow elsewhere, we can clear the buffer to keep memory
    // usage low
    buf.clear();
  }
  println!("got: {:?}", txt);
}
fn main() { parse_some_xml(); }

fn main_() {
  let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
  let reactor = tokio_core::reactor::Core::new().unwrap();
  RssBot::new(token, reactor.handle()).run(reactor);
}
