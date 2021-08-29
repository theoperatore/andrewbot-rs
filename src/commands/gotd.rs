use super::super::GotdMysqlStore;
use super::respond;
use chrono::{FixedOffset, Utc};
use cron::Schedule;
use serenity::{
  model::interactions::application_command::{
    ApplicationCommandInteraction, ApplicationCommandInteractionDataOptionValue,
  },
  prelude::Context,
  // utils::Colour,
};
use std::str::FromStr;
use std::sync::Arc;
use tracing::{error, info};

// use crate::clients::gotd;
use crate::store::model::NewGotdJob;
use crate::store::storage::GotdDb;

pub async fn handler(
  ctx: Arc<Context>,
  db: &GotdMysqlStore,
  command: &ApplicationCommandInteraction,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  let options = command
    .data
    .options
    .get(0)
    .expect("Expected time")
    .resolved
    .as_ref()
    .expect("Expected time str");

  if let ApplicationCommandInteractionDataOptionValue::String(time_of_day) = options {
    // default case is "night"
    let cron_schedule = match time_of_day.as_str() {
      // "morning" => String::from("0 8 18 * * * *"),
      "morning" => String::from("0 0 8 * * * *"),
      "noon" => String::from("0 0 12 * * * *"),
      _ => String::from("0 0 20 * * * *"),
    };

    let job = NewGotdJob {
      channel_id: *command.channel_id.as_u64(),
      guild_id: *command.guild_id.unwrap_or_default().as_u64(),
      cron_schedule: cron_schedule.clone(),
      created_by_id: *command.user.id.as_u64(),
    };

    // check if a sched exists; if it does report it!
    let sched = db.get_active_sched(command.channel_id.0)?;
    if sched.is_some() {
      let s = sched.unwrap();
      let schedule = Schedule::from_str(&s.cron_schedule).unwrap();
      let tz = FixedOffset::west(5 * 3600);
      let datetime = schedule.upcoming(tz).take(1).next().unwrap();
      let now = Utc::now().with_timezone(&tz);
      let diff = datetime - now;

      let msg = format!(
        "Gotd already set up for this channel. Next game sending on {}, ({}) mins",
        datetime,
        diff.num_minutes()
      );
      respond(&ctx, command, String::from(msg)).await?;
      return Ok(());
    }

    if let Err(why) = db.save_sched(job) {
      error!("Failed to insert data {}", why);
      return Err(why);
    };

    info!(
      "User {} created GotdJob for channel {} with sched {}",
      command.user.id, command.channel_id, cron_schedule
    );

    respond(
      &ctx,
      command,
      String::from(format!("Gotcha, scheduling for {}", time_of_day).as_str()),
    )
    .await?;
  } else {
    respond(&ctx, command, String::from("Not a valid time of day")).await?;
  }

  Ok(())
}
