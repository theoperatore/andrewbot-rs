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
        interactions::{Interaction, InteractionResponseType},
    },
    prelude::{Client, Context, EventHandler},
    utils::Colour,
};
use tracing::{error, info, instrument};

mod clients;
mod commands;
use clients::gotd;
use commands::ping::*;

struct Handler;

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
                        .description("Return a random Game of the Day from GiantBomb")
                })
                .await
            {
                error!("Cannot create guild command {}", why);
            }

            info!("Slash commands ready in guild {}", guild.id());
        }
    }

    // #[instrument(skip(self, ctx))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let cmd = &command.data.name;
            let usr = &command.user.name;
            info!("Got slash command '{}' by user '{}'", cmd, usr);

            let response = match command.data.name.as_str() {
                "gotd" => "Searching for game...",
                _ => "Not Implemented :(",
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |res| {
                    res.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|msg| msg.content(response))
                })
                .await
            {
                error!("Failed to respond to slash command: {}", why);
            };

            // show to the users that andrew bot is thinking...
            let typing = command.channel_id.start_typing(&ctx.http);
            if command.data.name.as_str() == "gotd" {
                match gotd::get_random_game().await {
                    Ok(game) => {
                        let img = gotd::parse_image(&game);
                        let date = gotd::parse_date(&game);
                        if let Err(why) = command
                            .channel_id
                            .send_message(&ctx.http, |m| {
                                m.content("Random Game");
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
            };

            match typing {
                Ok(t) => t.stop(),
                Err(err) => {
                    error!("Failed to show typing {}", err);
                    None
                }
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
        .event_handler(Handler)
        .application_id(app_id)
        .framework(framework)
        .await
        .expect("Err creating client");

    // Shard count is equal to the number of guilds? other forums say that sharding isn't
    // needed until you are at 2k guilds, but the serentiy code examples say sharding
    // is a good idea once your bot is on 2 or more servers. so :shrug: I'll just use
    // it and see what happens.
    if let Err(why) = client.start_shards(2).await {
        error!("Client error: {:?}", why);
    }
}
