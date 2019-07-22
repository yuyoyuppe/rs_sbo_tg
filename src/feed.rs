use chrono::{
  naive::NaiveDateTime,
  prelude::{DateTime, FixedOffset, Utc}
};
use curl::easy::Easy;
use failure::{err_msg, Error, ResultExt};
use quick_xml::{events::Event, Reader};
use std::{
  fs::File,
  io::{prelude::*, BufReader, Write},
  path::Path
};

#[derive(Debug)]
pub struct FeedItem {
  pub title:            String,
  pub url:              String,
  pub publication_date: DateTime<Utc>
}

#[derive(Debug, Default)]
pub struct Feed {
  pub url:   String,
  pub items: Vec<FeedItem>
}

impl Default for FeedItem {
  fn default() -> Self {
    Self {
      title:            Default::default(),
      url:              Default::default(),
      publication_date: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
    }
  }
}

pub fn download_test_feeds(feeds_file: &Path) -> Result<(), Error> {
  let feeds_file_path = feeds_file.parent().ok_or(err_msg("invalid feeds filepath!"))?;
  let feed_urls = BufReader::new(File::open(feeds_file)?);
  for (n, url) in feed_urls.lines().enumerate() {
    let output_filename = feeds_file_path.join("feeds").join(format! {"feed{}.xml", n});
    let mut easy = Easy::new();
    easy.follow_location(true)?;
    let url = url?;
    println!("processing {}...", url);
    easy.url(&url)?;
    let mut bytes: Vec<u8> = Vec::new();
    {
      let mut transfer = easy.transfer();
      transfer.write_function(|data| {
        bytes.extend_from_slice(data);
        Ok(data.len())
      })?;
      transfer.perform()?;
    }
    File::create(&output_filename)?.write_all(&bytes)?;
  }
  Ok(())
}

enum FeedItemField {
  Title,
  URL,
  PublicationDate,
  Invalid
}
pub fn test_feeds(feeds_path: &Path) -> Result<(), Error> {
  for path in
    std::fs::read_dir(feeds_path).context(format! {"{} is not a valid path!", feeds_path.display()})?
  {
    println!("feed: {:#?}", path);
    print_feed_items(path?.path().to_str().ok_or(err_msg("couldn't determine full path for feed item"))?)
  }
  Ok(())
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
      Ok(Event::Start(ref e)) => match e.name() {
        b"item" | b"entry" => {
          current_item = Some(FeedItem::default());
        }
        b"title" => {
          current_field = FeedItemField::Title;
        }
        b"link" => current_field = FeedItemField::URL,
        b"pubDate" | b"updated" | b"published" => current_field = FeedItemField::PublicationDate,
        _ => current_field = FeedItemField::Invalid
      },
      Ok(Event::Text(e)) | Ok(Event::CData(e)) => match current_field {
        FeedItemField::Title => {
          current_item =
            current_item.map(|i| FeedItem { title: e.unescape_and_decode(&reader).unwrap(), ..i });
        }
        FeedItemField::URL => {
          current_item = current_item.map(|i| FeedItem { url: e.unescape_and_decode(&reader).unwrap(), ..i });
        }
        FeedItemField::PublicationDate => {
          let datestr = e.unescape_and_decode(&reader).unwrap();
          let d = DateTime::<FixedOffset>::parse_from_rfc2822(&datestr)
            .or_else(|_| DateTime::<FixedOffset>::parse_from_rfc3339(&datestr))
            .unwrap() // TODO: this shit will fucking panic!
            .with_timezone(&Utc);
          current_item = current_item.map(|i| FeedItem { publication_date: d, ..i });
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
  println!("parsed {} items!", items.len());
  // for item in items {
  //   println!("item: {:?}", item);
  // }
}

fn parse_feed(raw_feed: &str) -> Result<Feed, Error> { Ok(Feed::default()) }
