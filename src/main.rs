use chrono::Utc;
use cron::Schedule;
use dotenv::dotenv;
use serenity::{
    async_trait,
    framework::standard::{
        macros::{group, hook},
        StandardFramework,
    },
    model::{
        channel::Message,
        event::ResumedEvent,
        gateway::Ready,
        id::{ChannelId, GuildId},
        interactions::{Interaction, InteractionResponseType},
    },
    prelude::{Client, Context, EventHandler /* TypeMapKey */},
    utils::Colour,
};
use std::str::FromStr;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
// use tokio::sync::RwLock;
use tracing::{error, info, instrument};

mod clients;
mod commands;
use clients::gotd;
use commands::ping::*;

// #[derive(Default)]
// struct Store;

// struct GotdStore;
// impl TypeMapKey for GotdStore {
//     type Value = Arc<RwLock<Store>>;
// }

struct Handler {
    is_loop_running: AtomicBool,
    dev_channel_id: u64,
}

#[async_trait]
impl EventHandler for Handler {
    // For instrument to work, all parameters must implement Debug.
    // Handler doesn't implement Debug here, so we specify to skip that argument.
    // Context doesn't implement Debug either, so it is also skipped.
    #[instrument(skip(self, _ctx))]
    async fn resume(&self, _ctx: Context, resume: ResumedEvent) {
        info!("Resumed; trace: {:?}", resume.trace);
    }

    // #[instrument(skip(self, ctx))]
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} ready and connected", ready.user.name);

        for guild in ready.guilds {
            if let Err(why) = guild
                .id()
                .create_application_command(&ctx.http, |cmd| {
                    cmd.name("gotd")
                        .description("Return a random Game of the Day from GiantBomb");
                    cmd.name("mem")
                        .description("Return stats on the cpu and memory")
                })
                .await
            {
                error!("Cannot create guild command {}", why);
            }

            info!("Slash commands ready in guild {}", guild.id());
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        info!("Cache ready. starting up loops");

        let actx = Arc::new(ctx);
        let dev_channel = self.dev_channel_id;
        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx2 = Arc::clone(&actx);
            tokio::spawn(async move {
                let expression = "30 * * * * * *";
                let schedule = Schedule::from_str(expression).unwrap();
                let mut datetime = schedule.upcoming(Utc).take(1).next().unwrap();
                info!("Next update at {}", datetime);
                loop {
                    if datetime < Utc::now() {
                        datetime = schedule.upcoming(Utc).take(1).next().unwrap();
                        log_system_load(Arc::clone(&ctx2), dev_channel).await;
                        info!("Next update at {}", datetime);
                    };
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            });
        }
    }

    // #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd = &command.data.name;
            let usr = &command.user.name;
            info!("Got slash command '{}' by user '{}'", cmd, usr);

            let ctx = Arc::new(ctx);
            if command.data.name.as_str() == "mem" {
                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |res| {
                        res.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| msg.content("Querying my bits..."))
                    })
                    .await
                {
                    error!("Failed to respond to slash command: {}", why);
                };

                let ctx1 = Arc::clone(&ctx);
                log_system_load(ctx1, *command.channel_id.as_u64()).await;
            } else if command.data.name.as_str() == "gotd" {
                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |res| {
                        res.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| msg.content("Searching for game..."))
                    })
                    .await
                {
                    error!("Failed to respond to slash command: {}", why);
                };

                if command.data.name.as_str() == "gotd" {
                    // show to the users that andrew bot is thinking...
                    let typing = command.channel_id.start_typing(&ctx.http);
                    match gotd::get_random_game().await {
                        Ok(game) => {
                            let img = gotd::parse_image(&game);
                            let date = gotd::parse_date(&game);
                            if let Err(why) = command
                                .channel_id
                                .send_message(&ctx.http, |m| {
                                    m.embed(|e| {
                                        let plats = game
                                            .platforms
                                            .map(|ps| {
                                                ps.iter()
                                                    .map(|p| p.name.clone())
                                                    .collect::<Vec<String>>()
                                                    .join(", ")
                                            })
                                            .unwrap_or(String::from("No platforms"));
                                        e.color(Colour::from(0x0099ff));
                                        e.title(game.name);
                                        e.author(|a| a.name("Game of the Day"));
                                        e.url(game.site_detail_url.unwrap_or(String::from("")));
                                        e.field("released", date, true);
                                        e.field("platforms", plats, true);
                                        e.description(game.deck.unwrap_or(String::from("")));
                                        e.image(img);
                                        e
                                    })
                                })
                                .await
                            {
                                error!("Failed to respond {}", why);
                                if let Err(why) = command
                                    .channel_id
                                    .say(&ctx.http, "Bzzzrt! Failed to find game.")
                                    .await
                                {
                                    error!("Failed to report failed response {}", why);
                                }
                            };
                        }
                        Err(err) => {
                            error!("Error fetching game {}", err);
                            if let Err(why) = command
                                .channel_id
                                .say(&ctx.http, "Bzzzrt! Failed to find game.")
                                .await
                            {
                                error!("Failed to report failed response {}", why);
                            }
                        }
                    }

                    match typing {
                        Ok(t) => t.stop(),
                        Err(err) => {
                            error!("Failed to show typing {}", err);
                            None
                        }
                    };
                };
            }
        }
    }
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
    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN env var required");
    let app_id: u64 = std::env::var("APPLICATION_ID")
        .expect("APPLICATION_ID env var required ")
        .parse()
        .expect("APPLICATION_ID not valid");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("~"))
        .before(before)
        .group(&GENERAL_GROUP);

    let dev_channel_id: u64 = 689814575306244110;

    let mut client = Client::builder(&token)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(true),
            dev_channel_id,
        })
        .application_id(app_id)
        .framework(framework)
        .await
        .expect("Err creating client");

    // {
    //     let mut data = client.data.write().await;
    //     data.insert::<GotdStore>(Arc::new(RwLock::new(Store::default())));
    // }

    // Shard count is equal to the number of guilds? other forums say that sharding isn't
    // needed until you are at 2k guilds, but the serentiy code examples say sharding
    // is a good idea once your bot is on 2 or more servers. so :shrug: I'll just use
    // it and see what happens.
    if let Err(why) = client.start_shards(2).await {
        error!("Client error: {:?}", why);
    };
}
