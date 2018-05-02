#![allow(unknown_lints, dead_code, unused_variables, unused_macros)]
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

use chrono::{naive::NaiveDateTime,
             prelude::{FixedOffset, Utc},
             DateTime};
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

#[derive(Debug)]
pub struct FeedItem {
  pub title:            String,
  pub url:              String,
  pub publication_date: DateTime<Utc>
}

impl Default for FeedItem {
  fn default() -> Self {
    FeedItem {
      title:            Default::default(),
      url:              Default::default(),
      publication_date: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
    }
  }
}
enum FeedItemField {
  Title,
  URL,
  PublicationDate,
  Invalid
}
fn test_feeds() {
  let mut p = get_home();
  p.push("feeds");
  for path in std::fs::read_dir(p).unwrap() {
    println!("feed: {:#?}", path);
    print_feed_items(path.unwrap().path().to_str().unwrap())
  }
}

fn print_feed_items(fname: &str) {
  let mut reader = Reader::from_file(fname).unwrap();
  reader.trim_text(true);

  let mut buf = Vec::new();

  let mut items: Vec<FeedItem> = Vec::new();
  let mut current_item: Option<FeedItem> = None;
  let mut current_field = FeedItemField::Invalid;
  loop {
    match reader.read_event(&mut buf) {
      // for triggering namespaced events, use this instead:
      // match reader.read_namespaced_event(&mut buf) {
      Ok(Event::Start(ref e)) => {
        // for namespaced:
        // Ok((ref namespace_value, Event::Start(ref e)))
        match e.name() {
          b"item" | b"entry" => {
            current_item = Some(Default::default());
          }
          b"title" => {
            current_field = FeedItemField::Title;
          }
          b"link" => current_field = FeedItemField::URL,
          b"pubDate" | b"updated" | b"published" => current_field = FeedItemField::PublicationDate,
          _ => current_field = FeedItemField::Invalid
        }
      }
      Ok(Event::Text(e)) | Ok(Event::CData(e)) => match current_field {
        FeedItemField::Title => {
          current_item = current_item.map(|i| FeedItem {
            title: e.unescape_and_decode(&reader).unwrap(),
            ..i
          });
        }
        FeedItemField::URL => {
          current_item = current_item.map(|i| FeedItem {
            url: e.unescape_and_decode(&reader).unwrap(),
            ..i
          });
        }
        FeedItemField::PublicationDate => {
          let datestr = e.unescape_and_decode(&reader).unwrap();
          let d = DateTime::<FixedOffset>::parse_from_rfc2822(&datestr)
            .or_else(|_| DateTime::<FixedOffset>::parse_from_rfc3339(&datestr))
            .unwrap() // TODO: this shit will fucking panic!
            .with_timezone(&Utc);
          current_item = current_item.map(|i| FeedItem {
            publication_date: d,
            ..i
          });
        }
        _ => ()
      },
      Ok(Event::Eof) => break,
      Err(e) => println!("Error at position {}: {:?}", reader.buffer_position(), e),
      Ok(Event::End(ref e)) => match e.name() {
        b"item" | b"entry" => {
          if let Some(i) = current_item {
            items.push(i);
            current_item = None;
          } else {
            println!("found </item> w/o <item>!");
          }
        }
        _ => ()
      },
      _ => ()
    }

    buf.clear();
  }
  for item in items {
    println!("item: {:?}", item);
  }
}
fn main() { test_feeds(); }

fn main_() {
  let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
  let reactor = tokio_core::reactor::Core::new().unwrap();
  RssBot::new(token, reactor.handle()).run(reactor);
}
