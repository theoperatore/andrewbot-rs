use super::model::{GotdJob, NewGotdJob};
use super::schema::gotd_schedules::dsl::{channel_id, gotd_schedules, guild_id, id, is_deleted};
use super::storage::GotdDb;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use std::error::Error;
use tracing::error;

#[derive(Clone)]
pub struct GotdMysqlStore {
  db: Pool<ConnectionManager<MysqlConnection>>,
}

impl GotdMysqlStore {
  pub fn new(db: Pool<ConnectionManager<MysqlConnection>>) -> Self {
    Self { db }
  }
}

impl GotdDb for GotdMysqlStore {
  /**
   * save a new cron schedule
   */
  fn save_sched(&self, job: NewGotdJob) -> Result<(), Box<dyn Error + Send + Sync>> {
    let conn = self.db.get()?;
    if let Err(why) = diesel::insert_into(gotd_schedules)
      .values(&job)
      .execute(&conn)
    {
      error!("Failed to insert data {}", why);
      return Err(Box::new(why));
    };

    Ok(())
  }

  /**
   * Check whether or not the current channel has an active sched or not
   * will also return false if there is no saved sched for this channel
   */
  fn has_active_sched(&self, channel: u64) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let conn = self.db.get()?;
    let results = gotd_schedules
      .filter(is_deleted.eq(false))
      .filter(channel_id.eq(channel))
      .limit(1)
      .load::<GotdJob>(&conn)?;

    Ok(!results.is_empty())
  }

  /**
   * If an active sched exists for channel, return it, otherwise
   * return an empty Option
   */
  fn get_active_sched(
    &self,
    channel: u64,
  ) -> Result<Option<GotdJob>, Box<dyn Error + Send + Sync>> {
    let conn = self.db.get()?;
    let results = gotd_schedules
      .filter(is_deleted.eq(false))
      .filter(channel_id.eq(channel))
      .limit(1)
      .load::<GotdJob>(&conn)?;

    if results.is_empty() {
      return Ok(None);
    }

    // because GotdJob doesn't implement the Copy/Clone traits
    // and I don't know why it can't (tried using derive(copy, clone))
    // then just map over the struct myself
    // alternatively I could try to return a borrowed reference
    // but then I think I'd have to mess with struct lifetimes and
    // I don't really want to do that...
    match results.get(0) {
      Some(g) => Ok(Some(GotdJob {
        id: g.id,
        channel_id: g.channel_id,
        guild_id: g.guild_id,
        cron_schedule: g.cron_schedule.clone(),
        created_on_ts: g.created_on_ts,
        created_by_id: g.created_by_id,
        is_deleted: g.is_deleted,
      })),
      None => Ok(None),
    }
  }

  fn get_all_active_sched_for_guild(
    &self,
    guild: u64,
  ) -> Result<Vec<GotdJob>, Box<dyn Error + Send + Sync>> {
    let conn = self.db.get()?;
    let results = gotd_schedules
      .filter(is_deleted.eq(false))
      .filter(guild_id.eq(guild))
      .load::<GotdJob>(&conn)?;

    Ok(results)
  }

  /**
   * Delete the sched identified by id. Return true if delete is sucessful.
   */
  fn delete_sched(&self, job_id: i32) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let conn = self.db.get()?;
    match diesel::update(gotd_schedules.filter(id.eq(job_id)))
      .set(is_deleted.eq(true))
      .execute(&conn)
    {
      Ok(num_updated) => Ok(num_updated == 1),
      Err(why) => Err(Box::new(why)),
    }
  }
}
