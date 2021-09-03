use super::model::{GotdJob, NewGotdJob};
use std::error::Error;

pub trait GotdDb {
  /**
   * save a new cron schedule
   */
  fn save_sched(&self, job: NewGotdJob) -> Result<(), Box<dyn Error + Send + Sync>>;

  /**
   * Check whether or not the current channel has an active sched or not
   * will also return false if there is no saved sched for this channel
   */
  fn has_active_sched(&self, channel_id: u64) -> Result<bool, Box<dyn Error + Send + Sync>>;

  /**
   * If an active sched exists for channel, return it, otherwise
   * return an empty Option
   */
  fn get_active_sched(
    &self,
    channel_id: u64,
  ) -> Result<Option<GotdJob>, Box<dyn Error + Send + Sync>>;

  /**
   * If an active sched exists for guild, return it, otherwise
   * return an empty Option
   */
  fn get_all_active_sched_for_guild(
    &self,
    guild_id: u64,
  ) -> Result<Vec<GotdJob>, Box<dyn Error + Send + Sync>>;

  /**
   * Get all active sched
   */
  fn get_all_active_sched(&self) -> Result<Vec<GotdJob>, Box<dyn Error + Send + Sync>>;

  /**
   * Delete the sched identified by id. Return true if delete is sucessful.
   */
  fn delete_sched(&self, id: i32) -> Result<bool, Box<dyn Error + Send + Sync>>;
}
