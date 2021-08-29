-- Your SQL goes here

CREATE TABLE gotd_schedules(
  id INT NOT NULL AUTO_INCREMENT,
  channel_id BIGINT UNSIGNED NOT NULL,
  guild_id BIGINT UNSIGNED NOT NULL,
  cron_schedule VARCHAR(255) NOT NULL,
  created_on_ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  created_by_id BIGINT UNSIGNED NOT NULL,
  is_deleted BOOLEAN NOT NULL DEFAULT false,
  PRIMARY KEY (id)
);

CREATE INDEX channel_id_index ON gotd_schedules(channel_id);

CREATE INDEX guild_id_index ON gotd_schedules(guild_id);

CREATE INDEX guild_channel_id_index ON gotd_schedules(channel_id, guild_id);
