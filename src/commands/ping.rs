use serenity::{
  framework::standard::{macros::command, CommandResult},
  model::channel::Message,
  prelude::Context,
};

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
  msg.channel_id.say(&ctx.http, "Pong! :)").await?;
  Ok(())
}
