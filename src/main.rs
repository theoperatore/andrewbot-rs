#[macro_use]
extern crate diesel;

mod clients;
mod commands;
mod store;

// use chrono::Utc;
// use cron::Schedule;
use dotenv::dotenv;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{group, hook},
        StandardFramework,
    },
    model::{
        channel::{Message, ReactionType},
        event::ResumedEvent,
        gateway::Ready,
        guild::Guild,
        id::{ChannelId, GuildId},
        interactions::{
            application_command::{ApplicationCommand, ApplicationCommandOptionType},
            Interaction,
        },
    },
    prelude::{Client, Context, EventHandler},
};
// use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use tracing::{error, info, instrument};

use chrono::{FixedOffset, Utc};
use commands::ping::*;
use cron::Schedule;
use std::str::FromStr;
use store::model::GotdJob;
use store::mysql_store::GotdMysqlStore;
use store::storage::GotdDb;
use tokio::sync::RwLock;

struct Job {
    job: GotdJob,
    schedule: Schedule,
    pub tz: FixedOffset,
    next_date: chrono::DateTime<FixedOffset>,
}

impl Job {
    pub fn new(job: GotdJob) -> Self {
        let schedule = Schedule::from_str(job.cron_schedule.as_str()).unwrap();
        let tz = FixedOffset::west(5 * 3600);
        let next_date = schedule.upcoming(tz).take(1).next().unwrap();
        Self {
            job,
            schedule,
            next_date,
            tz,
        }
    }

    pub fn channel_id(&self) -> u64 {
        self.job.channel_id
    }

    pub fn get_date(&self) -> chrono::DateTime<FixedOffset> {
        self.next_date
    }

    pub fn advance(&mut self) {
        self.next_date = self.schedule.upcoming(self.tz).take(1).next().unwrap();
    }
}

struct Handler {
    is_loop_running: AtomicBool,
    db: Arc<GotdMysqlStore>,
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, _ctx: Context, guild: Guild, is_new: bool) {
        info!(
            "AndrewBot added to guild: {} (is new: {})",
            guild.name, is_new
        );
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        let kevin_toms = "<:KevinToms:776453874310709249>";
        if new_message.content.contains(kevin_toms) {
            if let Err(why) = new_message
                .react(&ctx.http, ReactionType::from_str(kevin_toms).unwrap())
                .await
            {
                error!("Failed to react with Kevin Toms to Kevin Toms: {}", why);
            }
        }
    }

    // For instrument to work, all parameters must implement Debug.
    // Handler doesn't implement Debug here, so we specify to skip that argument.
    // Context doesn't implement Debug either, so it is also skipped.
    // #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        info!("Connection resumed; trace: {:?}", resume.trace);
    }

    // #[instrument(skip(self, ctx))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} ready", ready.user.name);

        if let Err(why) =
            ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
                commands
                    .create_application_command(|cmd| {
                        cmd.name("game")
                            .description("Return a random Game of the Day from GiantBomb")
                    })
                    .create_application_command(|cmd| {
                        cmd.name("gotd")
                            .description("Schedule a random game be send to this channel each day")
                            .create_option(|option| {
                                option
                                    .name("time")
                                    .description("When to send the game to the channel")
                                    .kind(ApplicationCommandOptionType::String)
                                    .required(true)
                                    .add_string_choice("Morning, usually around 8am EST", "morning")
                                    .add_string_choice("Midday, usually around 12pm EST", "noon")
                                    .add_string_choice("Evening, usually around 8pm EST", "night")
                            })
                    })
                    .create_application_command(|cmd| {
                        cmd.name("gotd-stop")
                            .description("Stop pulling a Game of the Day")
                    })
                    .create_application_command(|cmd| {
                        cmd.name("mem")
                            .description("Return stats on the cpu and memory")
                    })
            })
            .await
        {
            error!("Failed to register global application commands: {}", why);
        }

        info!("Registered slash commands");

        // DELETES ALL GUILD SLASH COMMANDS
        // for guild in ready.guilds {
        //     let commands = guild
        //         .id()
        //         .get_application_commands(&ctx.http)
        //         .await
        //         .ok()
        //         .unwrap_or_default();

        //     for command in commands {
        //         info!("deleting command {:?}", command);
        //         if let Err(why) = guild
        //             .id()
        //             .delete_application_command(&ctx.http, command.id)
        //             .await
        //         {
        //             error!("error deleting command {}", why);
        //         }
        //     }
        // }

        // THIS IS GOOD FOR TESTING
        // for guild in ready.guilds {
        //     if let Err(why) = guild
        //         .id()
        //         .create_application_command(&ctx.http, |cmd| {
        //             cmd.name("game")
        //                 .description("Return a random Game of the Day from GiantBomb")
        //         })
        //         .await
        //     {
        //         error!("Cannot create guild command {}", why);
        //     }

        //     if let Err(why) = guild
        //         .id()
        //         .create_application_command(&ctx.http, |cmd| {
        //             cmd.name("gotd")
        //                 .description("Schedule a random game be send to this channel each day")
        //                 .create_option(|option| {
        //                     option
        //                         .name("time")
        //                         .description("When to send the game to the channel")
        //                         .kind(ApplicationCommandOptionType::String)
        //                         .required(true)
        //                         .add_string_choice(
        //                             "Some time in the morning, usually around 8am EST",
        //                             "morning",
        //                         )
        //                         .add_string_choice(
        //                             "Some time around midday, usually around 12pm EST",
        //                             "noon",
        //                         )
        //                         .add_string_choice(
        //                             "Some time in the evening, usually around 8pm EST",
        //                             "night",
        //                         )
        //                 })
        //         })
        //         .await
        //     {
        //         error!("Cannot create guild command {}", why);
        //     }

        //     if let Err(why) = guild
        //         .id()
        //         .create_application_command(&ctx.http, |cmd| {
        //             cmd.name("mem")
        //                 .description("Return stats on the cpu and memory")
        //         })
        //         .await
        //     {
        //         error!("Cannot create guild command {}", why);
        //     }

        //     info!("Slash commands ready in guild {}", guild.id());
        // }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache ready");

        if self.is_loop_running.load(Ordering::Relaxed) {
            info!("Threads already running");
            return;
        }

        self.is_loop_running.store(true, Ordering::Relaxed);

        let db = Arc::clone(&self.db);
        let adb = Arc::clone(&db);
        let scheds: Arc<RwLock<Vec<Job>>> = Arc::new(RwLock::new(Vec::new()));

        // DYNAMIC SCHEDULING THREAD
        let ascheds = Arc::clone(&scheds);
        tokio::spawn(async move {
            info!("Starting db lookup thread");
            loop {
                let mut out = Vec::new();
                match db.get_all_active_sched() {
                    Ok(crons) => out.extend(crons.into_iter().map(|c| Job::new(c))),
                    Err(why) => error!("Failed to get crons for guild: {}", why),
                };

                // this needs to be in it's own closure in order to release the write
                // lock when it goes out of scope. otherwise it very rarely goes out of
                // scope and makes the gotd schedule thread hang.
                {
                    let mut v = ascheds.write().await;
                    *v = out;
                }
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            }
        });

        // GOTD THREAD
        let ascheds = Arc::clone(&scheds);
        let ctx = Arc::new(ctx);
        let actx = Arc::clone(&ctx);
        tokio::spawn(async move {
            info!("Starting gotd schedule thread");

            loop {
                let mut jobs = ascheds.write().await;
                for job in jobs.iter_mut() {
                    let adb = Arc::clone(&adb);
                    let datetime = job.get_date();
                    let now = Utc::now().with_timezone(&job.tz);

                    if datetime < now {
                        job.advance();
                        if let Err(why) =
                            commands::game::send_gotd(Arc::clone(&actx), adb, job.channel_id())
                                .await
                        {
                            error!("Failed to cron {}", why);
                        }
                    }
                }

                // todo: is there a way to sleep until it's time to execute the next
                // job? Then this doesn't have to run every 500 ms
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
        });
    }

    // #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd = &command.data.name;
            let usr = &command.user.name;
            match command.guild_id.unwrap().to_partial_guild(&ctx.http).await {
                Ok(guild) => info!(
                    "Command '{}' by user '{}' from guild '{}'",
                    cmd, usr, guild.name
                ),
                Err(_) => info!("Command '{}' by user '{}'", cmd, usr),
            };

            let actx = Arc::new(ctx);
            let actxc = Arc::clone(&actx);
            if let Err(why) = commands::handler(actx, &self.db, &command).await {
                let ctx_clone = Arc::clone(&actxc);
                error!("Failed to handle to command: {}", why);
                if let Err(why_cmd) = commands::respond(
                    &ctx_clone,
                    &command,
                    String::from("Bzzzrrt! Failed that command"),
                )
                .await
                {
                    error!("Failed to respond to command: {}", why_cmd);
                };
            };
        }
    }
}

#[hook]
#[instrument]
async fn before(_: &Context, msg: &Message, cmd: &str) -> bool {
    info!("Got command '{}' by user '{}'", cmd, msg.author.name);
    true
}

#[group]
#[commands(ping)]
struct General;

#[tokio::main]
#[instrument]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var required");

    let manager = ConnectionManager::<MysqlConnection>::new(db_url);
    let pool = diesel::r2d2::Pool::new(manager).unwrap();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env var required");
    let app_id: u64 = std::env::var("APPLICATION_ID")
        .expect("APPLICATION_ID env var required ")
        .parse()
        .expect("APPLICATION_ID not valid");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .before(before)
        .group(&GENERAL_GROUP);

    let mut client = Client::builder(&token)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
            db: Arc::new(GotdMysqlStore::new(pool)),
        })
        .application_id(app_id)
        .framework(framework)
        .await
        .expect("Err creating client");

    let dev_channel_id: u64 = 689814575306244110;
    if let Err(why) = ChannelId(dev_channel_id)
        .send_message(&client.cache_and_http.http, |m| m.content("I'm alive!"))
        .await
    {
        error!("Failed to send startup message: {}", why);
    }

    // Shard count is equal to the number of guilds? other forums say that sharding isn't
    // needed until you are at 2k guilds, but the serentiy code examples say sharding
    // is a good idea once your bot is on 2 or more servers. so :shrug: I'll just use
    // it and see what happens.
    if let Err(why) = client.start_autosharded().await {
        error!("Client error: {:?}", why);
    };
}
