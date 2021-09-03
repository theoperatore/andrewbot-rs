use super::super::GotdMysqlStore;
use super::respond;
use crate::store::storage::GotdDb;
use serenity::{
  model::interactions::application_command::ApplicationCommandInteraction, prelude::Context,
};
use std::sync::Arc;
use tracing::error;

pub async fn handler(
  ctx: Arc<Context>,
  db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let opt = db.get_active_sched(command.channel_id.0)?;

  if opt.is_none() {
    respond(
      &ctx,
      command,
      String::from("No active Game of the Day found for this channel."),
    )
    .await?;
    return Ok(());
  }

  let job = opt.unwrap();
  let did_delete = db.delete_sched(job.id)?;

  if did_delete {
    respond(&ctx, command, String::from("No more games for these days!")).await?;
  } else {
    respond(
      &ctx,
      command,
      String::from("Hmm. couldn't delete that for some reason..."),
    )
    .await?;
    error!(
      "Somehow found an active job but could not deactivate it {:?}",
      job.id
    );
  }

  Ok(())
}
