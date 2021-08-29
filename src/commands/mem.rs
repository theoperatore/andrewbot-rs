use super::super::GotdMysqlStore;
use super::respond;
use serenity::{
  model::{id::ChannelId, interactions::application_command::ApplicationCommandInteraction},
  prelude::Context,
};
use std::sync::Arc;
use tracing::error;

pub async fn handler(
  ctx: Arc<Context>,
  _db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  respond(&ctx, command, String::from("Querying my bits...")).await?;
  log_system_load(ctx, command.channel_id.0).await;
  Ok(())
}

async fn log_system_load(ctx: Arc<Context>, id: u64) {
  let cpu_load = sys_info::loadavg().unwrap();
  let mem_use = sys_info::mem_info().unwrap();

  // We can use ChannelId directly to send a message to a specific channel; in this case, the
  // message would be sent to the #testing channel on the discord server.
  if let Err(why) = ChannelId(id)
    .send_message(&ctx, |m| {
      m.embed(|e| {
        e.title("System Resource Load");
        e.field(
          "CPU Load Average",
          format!("{:.2}%", cpu_load.one * 10.0),
          false,
        );
        e.field(
          "Memory Usage",
          format!(
            "{:.2} MB / {:.2} MB ({:.2}% used)",
            mem_use.free as f32 / 1000.0,
            mem_use.total as f32 / 1000.0,
            (mem_use.free as f32 / 1000.0) / (mem_use.total as f32 / 1000.0)
          ),
          false,
        );
        e
      })
    })
    .await
  {
    error!("Error sending diognostic message: {:?}", why);
  };
}
