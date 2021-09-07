use crate::store::mysql_store::GotdMysqlStore;
use serde::Deserialize;
use serenity::{
  model::interactions::{
    application_command::ApplicationCommandInteraction, message_component::ButtonStyle,
    InteractionResponseType,
  },
  prelude::Context,
};
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
  let test = Wyr {
    id: String::from("2ec96e6e-0f56-11ec-99b3-f21898a01299"),
    title: String::from("WYR have Regeneration or Healing?"),
    url: String::from(
      "https://www.reddit.com/r/WouldYouRather/comments/iv5qqt/wyr_have_regeneration_or_healing/",
    ),
    options: vec![String::from("Regeneration"), String::from("Healing")],
  };
  Ok(test)
}
