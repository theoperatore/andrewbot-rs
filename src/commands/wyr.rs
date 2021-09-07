use crate::store::mysql_store::GotdMysqlStore;
use rand::Rng;
use serde::Deserialize;
use serde_json;
use serenity::{
  model::interactions::{
    application_command::ApplicationCommandInteraction, message_component::ButtonStyle,
    InteractionResponseType,
  },
  prelude::Context,
};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tracing::error;

#[derive(Deserialize)]
struct Wyr {
  id: String,
  title: String,
  url: String,
  options: Vec<String>,
}

pub async fn handler(
  ctx: Arc<Context>,
  _db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let typing = command.channel_id.start_typing(&ctx.http);
  let wyr = get_random().expect("Didn't get wyr");

  if let Err(why) = command
    .create_interaction_response(&ctx.http, |res| {
      res
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|m| {
          m.create_embed(|e| {
            e.title(&wyr.title)
              .url(&wyr.url)
              .description("from r/WouldYouRather")
          })
          .components(|c| {
            c.create_action_row(|a| {
              for opt in &wyr.options {
                a.create_button(|b| {
                  b.label(opt)
                    .custom_id(format!("wyr::{}::{}", opt, &wyr.id))
                    .style(ButtonStyle::Primary)
                });
              }
              a
            })
          })
        })
    })
    .await
  {
    error!("Failed to respond {}", why);
    command
      .channel_id
      .say(&ctx.http, "Bzzzrt! Failed to find a wyr.")
      .await?;
  }

  match typing {
    Ok(t) => t.stop(),
    Err(err) => {
      error!("Failed to show typing {}", err);
      None
    }
  };
  Ok(())
}

fn get_random() -> Result<Wyr, Box<dyn std::error::Error>> {
  let reader = BufReader::new(File::open("db/wyr/wyr-top-all.ndjson")?);

  // number between 0 and 678
  // wyr-top-all has a 678 lines...
  // this could be improved by collecting the number of lines, but that
  // would require reading the entire file into memory...I think?
  // this is good enough for now
  let idx = random(678);
  let line = match reader.lines().nth(idx) {
    Some(r) => match r {
      Ok(l) => l,
      Err(_) => String::from(""),
    },
    None => String::from(""),
  };

  let wyr: Wyr = serde_json::from_str(&line)?;
  Ok(wyr)
}

fn random(max: usize) -> usize {
  // get random int between 0 and (max - 1)
  rand::thread_rng().gen_range(0..max)
}
