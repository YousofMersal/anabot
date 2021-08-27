mod models;
use crate::models::*;

#[macro_use]
extern crate sqlx;

use dotenv;
use scheduler::JobScheduler;
#[allow(unused_imports)]
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        guild::Guild,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType, ApplicationCommandPermission,
                ApplicationCommandPermissionData, ApplicationCommandPermissionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::{collections::HashSet, env, sync::Arc};

// Main async function that imports env variables, and boots up the discord bot & the database.
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let (http, token) = establish_discord_connection();

    let (_owners, _bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        }

        Err(err) => panic!("Could not access application info: {:?}", err),
    };

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment")
        .parse()
        .expect("application id is not a valid id");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .await
        .expect("Error creating client");

    // Insert database pool into the global contex
    {
        let mut data = client.data.write().await;

        data.insert::<DB>(Arc::new(establish_db_connection().await));
        data.insert::<Schedule>(Arc::new(JobScheduler::new()));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    //Fires every time a command is called from discord
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        //let admins = vec!["181477689708904448", "224154198860759042"];
        let data = {
            let data_read = &ctx.data.read().await;

            data_read
                .get::<DB>()
                .expect("Something went wrong gettnig the database connection")
                .clone()
        };

        //Check which command was called and fire corresponding action
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "timer" => {
                    "Hey, I'm alive!".to_string()
                    //
                }
                "list" => {
                    let mut res = "Listing timers and their id's: \n".to_owned();
                    res = res.to_owned() + "-----------------\n";
                    if let Ok(db_res) = get_timers(&data).await {
                        if db_res.is_empty() {
                            res.clear();
                            res =
                                "No timers in the database yet! Use the /timer command to add one!"
                                    .to_string();
                        } else {
                            for timer in &db_res {
                                res = res + "id: " + &timer.id.to_string() + "\n";
                                res = res + "title: " + &timer.title.to_string() + "\n";
                                res = res + "recurring: " + &timer.recurring.to_string() + "\n";
                            }
                        };
                    } else {
                        res = "Something went wrong while getting timers,
                          try agian later.
                          If problem persists contact bot maintainer at yousof777@gmail.com
                          or on discord for support."
                            .to_string();
                    }
                    res
                }
                "delete" => {
                    //
                    "test".to_owned()
                }
                _ => "not implemented :(".to_string(),
            };

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        //let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {commands}).await;
        let pool = {
            let data_read = &ctx.data.read().await;

            data_read
                .get::<DB>()
                .expect("Something went wrong gettnig the database connection")
                .clone()
        };

        let res_timers = get_timers(&pool).await;

        if let Ok(timers) = res_timers {}

        ctx.set_presence(
            Some(serenity::model::gateway::Activity::playing(
                "The waiting game",
            )),
            serenity::model::user::OnlineStatus::Online,
        )
        .await;

        GuildId(326330465218985985)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("list")
                    .description("List all timers set for the bot")
                    .default_permission(true)
            })
            .await
            .unwrap();
        println!("{} is connected!", ready.user.name);
    }
}

fn establish_discord_connection() -> (serenity::http::Http, String) {
    let token = if let Ok(token) = env::var("DISCORD_TOKEN") {
        token
    } else {
        panic!("Expected DISCORD_TOKEN env variable \nEither set DISCORD_TOKEN env variable or create an .env file with DISCORD_TOKEN set like so: \nDISCORD_TOKEN=<TOKEN>\n")
    };

    (serenity::http::Http::new_with_token(&token), token)
}
