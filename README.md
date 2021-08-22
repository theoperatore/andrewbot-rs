# AndrewBot

A small Discord Bot written in Rust (so I can learn) that respondes to `/gotd` to post a random Game of the Day sourced from the [GiantBomb Api](https://www.giantbomb.com/api/).

## Adding to a discord server

Coming soon? Currently the Bot is in dev so it's private. Eventually maybe this will be easily added without having to host it yourself?

## Self-Hosted

If you wanna do it yourself:

1. [install rust](https://www.rust-lang.org/tools/install)
2. clone the code
3. Create a new [discord application](https://discord.com/developers/applications)
4. Add a bot in the "Bot settings"; Copy token. this is the `DISCORD_TOKEN` env variable
5. Go to "General Information" and copy the application id; Copy application id. This is the `APPLICATION_ID` env variable
6. Create a `.env` file (easiest) or use the CLI and add in `DISCORD_TOKEN=<paste your token>` and `APPLICATION_ID=<paste your app id>`.
7. Control log level with `RUST_LOG=info`; change info to "debug" if you want it all...
8. Complie and run with `cargo run`
9. One-time-setup: add your [bot to your server](https://discord.com/developers/docs/topics/oauth2#bots)
10. try typing a `~ping` into discord to see your bot answer with a `Pong :)` and the logs populate on the terminal
11. try typing `/gotd` to use the slash command.
12. profit!

# License

MIT
