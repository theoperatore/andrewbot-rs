pub mod ping;

use super::GotdMysqlStore;
use serenity::{
  model::interactions::{
    application_command::ApplicationCommandInteraction, InteractionResponseType,
  },
  prelude::Context,
};
use std::sync::Arc;
use tracing::error;

pub mod game;
mod gotd;
mod gotd_stop;
mod mem;

pub async fn handler(
  ctx: Arc<Context>,
  db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  match command.data.name.as_str() {
    "mem" => mem::handler(ctx, db, command).await?,
    "game" => game::handler(ctx, db, command).await?,
    "gotd" => gotd::handler(ctx, db, command).await?,
    "gotd-stop" => gotd_stop::handler(ctx, db, command).await?,
    _ => error!("Unknown slash command"),
  };

  Ok(())
}

pub async fn respond(
  ctx: &Arc<Context>,
  command: &ApplicationCommandInteraction,
  msg: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  command
    .create_interaction_response(&ctx.http, |res| {
      res
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|m| m.content(msg))
    })
    .await?;
  Ok(())
}
