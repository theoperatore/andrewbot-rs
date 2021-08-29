table! {
    gotd_schedules (id) {
        id -> Integer,
        channel_id -> Unsigned<Bigint>,
        guild_id -> Unsigned<Bigint>,
        cron_schedule -> Varchar,
        created_on_ts -> Nullable<Timestamp>,
        created_by_id -> Unsigned<Bigint>,
        is_deleted -> Bool,
    }
}
