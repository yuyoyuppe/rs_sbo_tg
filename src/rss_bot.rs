extern crate telegram_bot;
extern crate tokio_core;
extern crate url;

use domain_types::{Feed, User};
use futures::future::{err, Either};

use telegram_bot::{prelude::*, Api, Message, MessageEntityKind, MessageKind, ParseMode, UpdateKind};

use futures::{Future, Stream};
use tokio_core::reactor::{Core, Handle};

use std::{collections::HashMap, fmt::Debug};

pub struct RssBot {
  telegram_api:   Api,
  reactor_handle: Handle,
  users:          HashMap<telegram_bot::UserId, User>
}

pub trait TGFuture: Future<Item = Message, Error = telegram_bot::Error> {}
impl<F: Future<Item = Message, Error = telegram_bot::Error>> TGFuture for F {}

impl RssBot {
  pub fn run(&mut self, mut reactor: Core) {
    let future = self.telegram_api.stream().for_each(|update| {
      if let UpdateKind::Message(message) = update.kind {
        RssBot::dispatch(self, message)
      }
      Ok(())
    });
    reactor.run(future).unwrap();
  }

  pub fn new(telegram_token: String, reactor_handle: Handle) -> Self {
    let telegram_api = telegram_bot::Api::configure(telegram_token)
      .build(reactor_handle.clone())
      .unwrap();
    let users = HashMap::new();
    Self {
      telegram_api,
      reactor_handle,
      users
    }
  }

  fn dispatch(&mut self, message: Message) {
    enum Command {
      Start,
      Help,
      Stop,
      Add
    }
    let command = match message.kind {
      MessageKind::Text { ref data, .. } => {
        let s = data.as_str();
        if s == "/start" {
          Command::Start
        } else if s == "/stop" {
          Command::Stop
        } else if s.starts_with("/add") {
          Command::Add
        } else {
          Command::Help
        }
      }
      _ => Command::Help
    };
    match command {
      Command::Help => RssBot::schedule(self.show_help(&message), &self.reactor_handle),
      Command::Start => RssBot::schedule(self.register_user(&message), &self.reactor_handle),
      Command::Stop => RssBot::schedule(self.unregister_user(&message), &self.reactor_handle),
      Command::Add => RssBot::schedule(self.authorized(RssBot::add_feed, message), &self.reactor_handle)
    }
  }

  fn add_feed(&mut self, message: Message) -> impl TGFuture {
    let mut new_feeds = vec![];
    if let MessageKind::Text {
      ref entities,
      ref data
    } = message.kind
    {
      new_feeds = entities
        .iter()
        .filter(|e| e.kind == MessageEntityKind::Url)
        .map(|e| {
          let from_ = e.offset as usize;
          let to_ = from_ + e.length as usize;
          String::from(&data[from_..to_])
        })
        .filter(|url| match url::Url::parse(url) {
          Err(_) => false,
          Ok(parsed_url) => !parsed_url.cannot_be_a_base()
        })
        .map(|url| Feed { url })
        .collect();
    }
    let nadded = new_feeds.len();
    self
      .users
      .get_mut(&message.from.id)
      .unwrap()
      .feeds
      .append(&mut new_feeds);
    let reply = if nadded == 0 {
      String::from("Couldn't parse any feed URLs!")
    } else {
      format!("Successfully added {} feed URLs!", nadded)
    };
    self
      .telegram_api
      .send(message.chat.text(reply).parse_mode(ParseMode::Markdown))
  }

  fn show_help(&mut self, message: &Message) -> impl TGFuture {
    self.telegram_api.send(
      message
        .chat
        .text(
          r#"Hey there!
You can use one of the following commands:
`/start` - _initiate registration process(it's just a couple of questions_)
`/stop` - _unregister yourself and delete ALL your data_
`/list` - _show your feeds and their settings_
`/add <atom/feed URLs>` - _subscribe to feeds using their URLs_
`/del <feed IDs>` - _unsubscribe from feeds using their IDs_
    "#
        )
        .parse_mode(ParseMode::Markdown)
    )
  }

  fn authorized<F, R>(&mut self, f: F, message: Message) -> Either<impl TGFuture, impl TGFuture>
  where
    F: FnOnce(&mut Self, Message) -> R,
    R: TGFuture
  {
    if self.users.get(&message.from.id).is_some() {
      Either::A(f(self, message))
    } else {
      let api = self.telegram_api.clone();
      Either::B(
        err(telegram_bot::Error::from(""))
          .or_else(move |_| api.send(message.text_reply("Please authorize before sending this command!")))
      )
    }
  }

  fn unregister_user(&mut self, message: &Message) -> impl TGFuture {
    let reply = if self.users.remove(&message.from.id).is_some() {
      message.text_reply("It's painful to see you go. Godspeed you, though!")
    } else {
      message.text_reply("We haven't started anything yet 😜")
    };
    self.telegram_api.send(reply)
  }

  fn register_user(&mut self, message: &Message) -> impl TGFuture {
    let mut reply = if self.users.get(&message.from.id).is_some() {
      message.text_reply("You're already registered!")
    } else {
      self.users.insert(
        message.from.id,
        User {
          telegram_user_id: message.from.id,
          ..Default::default()
        }
      );
      message.text_reply("Successfully registered! Now you can add new feeds with `/add <atom/feed URLs>`")
    };
    self.telegram_api.send(reply.parse_mode(ParseMode::Markdown))
  }

  fn schedule<T, E: Debug, F: Future<Item = T, Error = E> + 'static>(f: F, handle: &Handle) {
    handle.spawn({
      f.map_err(|error| println!("well, here's that: {:?}", error))
        .map(|_msg| ())
    })
  }
}
