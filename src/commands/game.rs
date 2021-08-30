use super::super::GotdMysqlStore;
use super::respond;
use serenity::{
  model::{id::ChannelId, interactions::application_command::ApplicationCommandInteraction},
  prelude::Context,
  utils::Colour,
};
use std::sync::Arc;
use tracing::error;

use crate::clients::gotd;

pub async fn handler(
  ctx: Arc<Context>,
  _db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  respond(&ctx, command, String::from("Searching for game...")).await?;

  // show to the users that andrew bot is thinking...
  let typing = command.channel_id.start_typing(&ctx.http);
  match gotd::get_random_game().await {
    Ok(game) => {
      let img = gotd::parse_image(&game);
      let date = gotd::parse_date(&game);
      if let Err(why) = command
        .channel_id
        .send_message(&ctx.http, |m| {
          m.embed(|e| {
            let plats = game
              .platforms
              .map(|ps| {
                ps.iter()
                  .map(|p| p.name.clone())
                  .collect::<Vec<String>>()
                  .join(", ")
              })
              .unwrap_or(String::from("No platforms"));
            e.color(Colour::from(0x0099ff));
            e.title(game.name);
            e.author(|a| a.name("Game of the Day"));
            e.url(game.site_detail_url.unwrap_or(String::from("")));
            e.field("released", date, true);
            e.field("platforms", plats, true);
            e.description(game.deck.unwrap_or(String::from("")));
            e.image(img);
            e
          })
        })
        .await
      {
        error!("Failed to respond {}", why);
        command
          .channel_id
          .say(&ctx.http, "Bzzzrt! Failed to find game.")
          .await?;
      };
    }
    Err(err) => {
      error!("Error fetching game {}", err);
      command
        .channel_id
        .say(&ctx.http, "Bzzzrt! Failed to find game.")
        .await?;
    }
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

pub async fn send_gotd(
  ctx: Arc<Context>,
  _db: Arc<GotdMysqlStore>,
  channel_id: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let channel = ChannelId(channel_id);
  let typing = channel.start_typing(&ctx.http);
  match gotd::get_random_game().await {
    Ok(game) => {
      let img = gotd::parse_image(&game);
      let date = gotd::parse_date(&game);
      if let Err(why) = channel
        .send_message(&ctx.http, |m| {
          m.embed(|e| {
            let plats = game
              .platforms
              .map(|ps| {
                ps.iter()
                  .map(|p| p.name.clone())
                  .collect::<Vec<String>>()
                  .join(", ")
              })
              .unwrap_or(String::from("No platforms"));
            e.color(Colour::from(0x0099ff));
            e.title(game.name);
            e.author(|a| a.name("Game of the Day"));
            e.url(game.site_detail_url.unwrap_or(String::from("")));
            e.field("released", date, true);
            e.field("platforms", plats, true);
            e.description(game.deck.unwrap_or(String::from("")));
            e.image(img);
            e
          })
        })
        .await
      {
        error!("Failed to respond {}", why);
        channel
          .say(&ctx.http, "Bzzzrt! Failed to find game.")
          .await?;
      };
    }
    Err(err) => {
      error!("Error fetching game {}", err);
      channel
        .say(&ctx.http, "Bzzzrt! Failed to find game.")
        .await?;
    }
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