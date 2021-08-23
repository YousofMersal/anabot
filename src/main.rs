#[macro_use]
extern crate diesel;
mod models;
mod schema;

use self::models::*;
use diesel::prelude::*;
use dotenv;
#[allow(unused_imports)]
use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        guild::Guild,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType, ApplicationCommandPermission,
                ApplicationCommandPermissionData, ApplicationCommandPermissionType,
            },
            Interaction, InteractionApplicationCommandCallbackDataFlags, InteractionResponseType,
        },
        Permissions,
    },
    prelude::*,
};
use std::{collections::HashSet, env};

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

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        //let admins = vec!["181477689708904448", "224154198860759042"];

        let db_conn: PgConnection = establish_db_connection();
        if let Interaction::ApplicationCommand(command) = interaction {
            let content = match command.data.name.as_str() {
                "timer" => "Hey, I'm alive!".to_string(),
                "list" => {
                    let mut res = "Listing timers and their id's: \n".to_owned();
                    res = res.to_owned() + "-----------------\n";
                    let db_results = retrieve_timers(&db_conn);
                    for timer in &db_results {
                        res = res + "id: " + &timer.id.to_string() + "\n";
                        res = res + "title: " + &timer.title.to_string() + "\n";
                        res = res + "recurring: " + &timer.recurring.to_string() + "\n";
                    }

                    if db_results.is_empty() {
                        "No timers currently set".to_owned()
                    } else {
                        res
                    }
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
        println!("{} is connected!", ready.user.name);

        //let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {commands}).await;
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
    }
}

fn retrieve_timers(conn: &PgConnection) -> Vec<Timer> {
    use crate::schema::timers::dsl::*;

    timers
        .load::<Timer>(conn)
        .expect("Something Went wrong while retriveving timers")
}

fn establish_discord_connection() -> (serenity::http::Http, String) {
    let token = if let Ok(token) = env::var("DISCORD_TOKEN") {
        token
    } else {
        panic!("Expected DISCORD_TOKEN env variable \nEither set DISCORD_TOKEN env variable or create an .env file with DISCORD_TOKEN set like so: \nDISCORD_TOKEN=<TOKEN>\n")
    };

    (serenity::http::Http::new_with_token(&token), token)
}

fn establish_db_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url).expect("Error connection to db")
}
