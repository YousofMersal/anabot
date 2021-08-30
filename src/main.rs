mod db;
mod handler;
use crate::db::*;
use handler::*;

#[macro_use]
extern crate sqlx;

use dotenv;
use scheduler::{Job, JobScheduler};
#[allow(unused_imports)]
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        guild::{Guild, PartialGuild},
        id::{ChannelId, CommandId, GuildId},
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

use serenity::{
    http::{CacheHttp, Http},
    model::channel::ReactionType,
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

fn channel_raid_warn(http: Arc<&Http>, time: &Timer) {
    async {
        let channel = ChannelId(326349171940655105);
        let channel_res = channel.send_message(*http, |m| {
            m.add_embed(|e| {
                e.title(&time.title);
                if let Some(bdy) = &time.body {
                    e.description(&format!("{}\n", bdy).to_string());
                }
                if let Some(rl) = &time.raid_lead {
                    e.field("_Raid lead_", rl, false);
                };
                e.colour(serenity::utils::Colour::RED);
                e.footer(|f| {
                    f.text(
                        "click corresponding reaction.
✅: attending, ❌: not attending, ❔: tentative, ⌚: late",
                    )
                });
                e
            });
            m.reactions([
                ReactionType::Unicode("✅".to_owned()),
                ReactionType::Unicode("❌".to_owned()),
                ReactionType::Unicode("❔".to_owned()),
                ReactionType::Unicode("⌚".to_owned()),
            ]);
            m
        });

        channel_res.await
    };
}

fn establish_discord_connection() -> (serenity::http::Http, String) {
    let token = if let Ok(token) = env::var("DISCORD_TOKEN") {
        token
    } else {
        panic!("Expected DISCORD_TOKEN env variable \nEither set DISCORD_TOKEN env variable or create an .env file with DISCORD_TOKEN set like so: \nDISCORD_TOKEN=<TOKEN>\n")
    };

    (serenity::http::Http::new_with_token(&token), token)
}
