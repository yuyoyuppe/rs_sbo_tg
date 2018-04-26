extern crate telegram_bot;
extern crate tokio_core;

use domain_types::User;
use futures::future::{err, ok};

use telegram_bot::prelude::*;
use telegram_bot::{Api, Message, MessageKind, UpdateKind};

use futures::{Future, Stream};
use tokio_core::reactor::{Core, Handle};

use std::collections::HashMap;
use std::fmt::Debug;

pub struct RssBot {
  telegram_api: Api,
  reactor_handle: Handle,
  users: HashMap<telegram_bot::UserId, User>
}

// TODO(future): rewrite when #![feature(trait_alias)] is implemented
macro_rules! TGFuture {
  () => { impl Future<Item = Message, Error = telegram_bot::Error> }
}

impl RssBot {
  fn schedule<T, E: Debug, F: Future<Item = T, Error = E> + 'static>(f: F, handle: &Handle) {
    handle.spawn({
      f.map_err(|error| println!("well, here's that: {:?}", error))
        .map(|_msg| ())
    })
  }

  fn register_user(&mut self, message: Message) -> TGFuture!() {
    let reply = if self.users.get(&message.from.id).is_some() {
      message.text_reply("You're already registered!")
    } else {
      self.users.insert(
        message.from.id,
        User {
          telegram_user_id: message.from.id,
          ..Default::default()
        }
      );
      message.text_reply("Thank you! Now you can add new feeds etc.")
    };
    self.telegram_api.send(reply)
  }

  fn unregister_user(&mut self, message: Message) -> TGFuture!() {
    let reply = if self.users.remove(&message.from.id).is_some() {
      message.text_reply("It's painful to see you go. Godspeed you, though!")
    } else {
      message.text_reply("We haven't started anything yet 😜")
    };
    self.telegram_api.send(reply)
  }

  fn please_authorize(&mut self, message: Message) -> TGFuture!() {
    self
      .telegram_api
      .send(message.text_reply("Please authorize before sending the commands!"))
  }

  // fn authorized<F, R: Future<Item = Message, Error = telegram_bot::Error>>(
  //   &mut self,
  //   f: F,
  //   message: Message
  // ) -> F
  // where
  //   F: FnOnce(&mut Self, Message) -> R
  // {
  //   if self.users.get(&message.from.id).is_some() {
  //     f
  //   } else {
  //     err(telegram_bot::Error::from("@_@"))
  //   }
  // }
  fn authorized(&mut self, message: Message) -> TGFuture!() {
    if self.users.get(&message.from.id).is_some() {
      ok(message)
    } else {
      err(telegram_bot::Error::from(""))
    }
  }

  fn show_help(&mut self, message: Message) -> TGFuture!() {
    self.telegram_api.send(message.chat.text(
      r#"Hey there!
    You can use one of the following commands:
    /start - initiate registration process(it's just a couple of questions)
    /stop - unregister yourself and delete ALL your data
    /list - show your feeds and their settings
    /add <atom/feed URLs> - subscribe to feeds using their URLs
    /remove <feed IDs> - unsubscribe from feeds using their IDs
    "#
    ))
  }

  fn dispatch(&mut self, message: Message) {
    // TODO(future): remove that when non-lexical lifetimes are landed, so we can move 'message' w/o compiler error
    enum Command {
      Start,
      Help,
      Stop,
      Add
    }
    let command = match message.kind {
      MessageKind::Text { ref data, .. } => match data.as_str() {
        "/start" => Command::Start,
        "/stop" => Command::Stop,
        "/add" => Command::Add,
        _ => Command::Help
      },
      _ => Command::Help
    };
    let vseok = self.telegram_api.send(message.chat.text("VSE NORM"));
    let vsebad = self.telegram_api.send(message.chat.text("VSE PLOHO"));

    /*
    let test = self
      .authorized(message.clone())
      .or_else(|m| self.please_authorize(message.clone()))
      .and_then(|m| vseok)
      ;
    */
    match command {
      Command::Help => RssBot::schedule(self.show_help(message), &self.reactor_handle),
      Command::Start => RssBot::schedule(self.register_user(message), &self.reactor_handle),
      Command::Stop => RssBot::schedule(self.unregister_user(message), &self.reactor_handle),
      Command::Add => RssBot::schedule(self.unregister_user(message), &self.reactor_handle) /*RssBot::schedule(test, &self.reactor_handle)*/
    }
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

  pub fn run(&mut self, mut reactor: Core) {
    let future = self.telegram_api.stream().for_each(|update| {
      if let UpdateKind::Message(message) = update.kind {
        RssBot::dispatch(self, message)
      }
      Ok(())
    });
    reactor.run(future).unwrap();
  }
}
