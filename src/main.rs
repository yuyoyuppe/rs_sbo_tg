extern crate chrono;
extern crate futures;
extern crate telegram_bot;
extern crate tokio_core;

pub mod domain_types;

use std::env;
use std::fmt::Debug;

use futures::{Future, Stream};
use telegram_bot::prelude::*;
use telegram_bot::{Api, Message, MessageKind, ParseMode, UpdateKind};
use tokio_core::reactor::{Core, Handle};

fn schedule<T, E: Debug, F: Future<Item = T, Error = E> + 'static>(f: F, handle: &Handle) {
  handle.spawn({
    f.map_err(|error| println!("well, here's that: {:?}", error))
      .map(|_msg| ())
  })
}

fn test_message(api: Api, message: Message, handle: &Handle) {
  let simple = api.send(message.text_reply("Simple message"));

  let markdown = api.send(
    message
      .text_reply("`Markdown message`")
      .parse_mode(ParseMode::Markdown)
  );

  let html = api.send(
    message
      .text_reply("<b>Bold HTML message</b>")
      .parse_mode(ParseMode::Html)
  );

  schedule(simple.and_then(|_| markdown).and_then(|_| html), handle);
}

fn test_reply(api: Api, message: Message, handle: &Handle) {
  let msg = api.send(message.text_reply("Reply to message"));
  let chat = api.send(message.chat.text("Text to message chat"));

  let private = api.send(message.from.text("Private text"));
  schedule(msg.and_then(|_| chat).and_then(|_| private), handle);
}

fn show_help(api: Api, message: Message, handle: &Handle) {
  let help_msg = api.send(message.chat.text(
    r#"Hey there!
You can use one of the following commands:
/start - initiate registration process(it's just a couple of questions) 
/stop - unregister yourself and delete ALL your data
/list - show your feeds and their settings
/add <atom/feed URLs> - subscribe to feeds using their URLs 
/remove <feed IDs> - unsubscribe from feeds using their IDs
"#
  ));
  schedule(help_msg, handle);
}
fn register_user(api: Api, message: Message, handle: &Handle) {
  // check if we're actually rgstrd
  // 
}

fn dispatch(api: Api, message: Message, handle: &Handle) {
  let function: fn(Api, Message, &Handle) = match message.kind {
    MessageKind::Text { ref data, .. } => match data.as_str() {
      "/help" => test_message,
      "/reply" => test_reply,
      "/start" => register_user,
      _ => show_help
    },
    _ => return
  };

  function(api, message, handle)
}

fn main() {
  let token = env::var("TELEGRAM_BOT_TOKEN").unwrap();
  let mut core = Core::new().unwrap();
  let handle = core.handle();
  let api = Api::configure(token).build(core.handle()).unwrap();

  let future = api.stream().for_each(|update| {
    if let UpdateKind::Message(message) = update.kind {
      dispatch(api.clone(), message, &handle)
    }
    Ok(())
  });

  core.run(future).unwrap();
}
