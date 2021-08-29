use super::schema::gotd_schedules;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "gotd_schedules"]
#[primary_key("id")]
pub struct GotdJob {
  pub id: i32,
  pub channel_id: u64,
  pub guild_id: u64,
  pub cron_schedule: String,
  pub created_on_ts: Option<chrono::NaiveDateTime>,
  pub created_by_id: u64,
  pub is_deleted: bool,
}

#[derive(Insertable, Debug)]
#[table_name = "gotd_schedules"]
pub struct NewGotdJob {
  pub channel_id: u64,
  pub guild_id: u64,
  pub cron_schedule: String,
  pub created_by_id: u64,
}
