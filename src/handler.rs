#![allow(dead_code)]

use scheduler::Job;
use sqlx::PgPool;
use crate::db::*;


#[allow(unused_imports)]
use crate::channel_raid_warn;

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
                ApplicationCommandOptionType, ApplicationCommandInteractionDataOption
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    //Fires every time a command is called from discord
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
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
                    let mut res = "";
                    let command_options = &command.data.options;

                    let mut new_timer = NewTimer {
                        title: "placeholder".to_string(),
                        time: "placeholder".to_string(),
                        recurring: true,
                        ..Default::default()
                    };

                    for option in command_options {
                        match option.name.as_str() {
                            "title" => {
                                if let Some(val) = &option.value {
                                    new_timer.title = val.to_string();
                                }
                            }
                            "body" => {
                                if let Some(val) = &option.value {
                                    new_timer.body = Some(val.to_string());
                                }
                            }
                            "raidlead" => {
                                if let Some(val) = &option.value {
                                    new_timer.raid_lead = Some(val.to_string());
                                }
                            }
                            "time" => {
                                if let Some(val) = &option.value {
                                    let str = &val.to_string();
                                    match convert_string(&str) {
                                        Ok(tmp) => {
                                            println!("{}", tmp);
                                        }
                                        Err(e) => {println!("{:?}", e)},
                                    }

                                    let conversion = naive_convert(str);
                                    if let Ok(value) = conversion {
                                        new_timer.time = value;
                                    } else {
                                        res = "Time format malformed, for help use /timerhelp"
                                    }

                                }
                            }
                            "channel" => {
                                if let Some(opt) = &option.resolved {
                                    match opt {
                                        ApplicationCommandInteractionDataOptionValue::Channel(chan) => {
                                            match chan.kind {
                                                serenity::model::channel::ChannelType::Text => {
                                                    new_timer.channel = *chan.id.as_u64();
                                                },
                                                _ => {
                                                    res = "Please select a normal text channel when selecting channel to announce in.";
                                                }
                                            }
                                        },
                                        _ => eprintln!("unknown option found while createing new timer")

                                    }
                                }
                            }
                            _ => eprintln!("unknown option found while createing new timer")
                        }

                    }

                    if let Ok(_) = add_timer(&data, &new_timer).await {
                        {
                            let data_read = &ctx.data.write().await;

                            let schedule = data_read
                                .get::<Schedule>()
                                .expect("Something went wrong gettnig the database connection")
                                .clone();

                            let mut sched_lock = schedule.lock().await;

                            let mut time = "";

                            for option in command_options {
                                if option.name == "time" {
                                    if let Some(o_time) = &option.value {
                                        if let Some(s) = o_time.as_str() {
                                            time = s;
                                        }
                                    }
                                }
                            };

                            let job = Job::new(&naive_convert(&time).expect("could not convert"), move |uuid, _l| {
                                println!("{}", uuid);
                                channel_raid_warn(new_timer.clone());
                            });

                            
                            match job {
                                Ok(job) => {
                                    sched_lock.add(job).expect("error adding job to queue"); 
                                }
                                Err(e) => {
                                    println!("error adding job?: {}", e);
                                    panic!();
                                }
                            }
                        };

                        res = "Timer succesfully registered"
                    } 


                    res.to_string()
                }
                "timehelp" => "The time is a space seperated \"list\" of time units starting with the hour then minute then day, then optionally month and year.
                    Named options are Day and Month which can be capitalized of not, and have the first 3 letters of the day/month.
                    Resulting in a format that must be like so: **Hour Minute Day (Month) (Year)**;
                a valid option looks as so **19 30 Thu 2021** which will make a message every Thursday at 19:30 for the whole of 2021.
                    **Parentheses** denotes _optional_ time units, omitting an optional unit will default to doing it at every year/month.
                    If multiple time slots are wanted a comma seperated list is also possible, bigger spans a \"-\" span can be used.
                    To return to last example if for example the same timer needs to be fired twice on multiple days at the same time,
                    it can be written like so: **19,18 30 Wed,Fri 6,7** which will send a message at 18:30 and 19:30 every wednesday and friday in the months of june and july."
                        .to_owned(),
                    "list" => {
                        let mut res = "Listing timers and their id's: \n".to_owned();
                        res = res.to_owned() + "---------\n";
                        if let Ok(db_res) = get_timers(&data).await {
                            if db_res.is_empty() {
                                res.clear();
                                res =
                                    "No timers in the database yet! Use the `/timer` command to add one!\nTo get help with creating a time the use the `/timerhelp` command"
                                    .to_string();
                                } else {
                                    for timer in &db_res {
                                        res = res + "id: " + &timer.id.to_string() + "\n";
                                        res = res + "title: " + &timer.title.to_string() + "\n";
                                        //TODO: Print out what time the timer is fired
                                        //res = res + "recurring: " + &timer.recurring.to_string() + "\n";
                                        res = res + "time of fire: " + &timer.time.to_string() + "\n";
                                        res = res + "------";
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
                "delete_timer" => {
                    // TODO: Also delete timer from currently running process.
                    // probably by addin UUID from job to database entry, 
                    // and useing sched_lock.remove(uuid) to do so
                    let command_options = &command.data.options; 
                    delete_timer(command_options, &data).await
                }
                _ => "not implemented :(".to_string(),
            };

            let m_split = split_to_discord_size(content.clone());
            let f_split = m_split.first();

            if let Some(f_string) = f_split {
                if let Err(why) = command
                    .create_interaction_response(&ctx.http, |response| {
                        response
                            .kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|message| message.content(f_string))
                    })
                .await {
                    println!("Cannot respond to slash command: {}", why);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        //let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {commands}).await;
        let _pool = {
            let data_read = &ctx.data.read().await;

            data_read
                .get::<DB>()
                .expect("Something went wrong getting the database connection")
                .clone()
        };


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

        GuildId(326330465218985985)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("timehelp")
                    .description("Get a detailed description of oh the timer format works")
                    .default_permission(true)
            })
        .await
            .unwrap();

        GuildId(326330465218985985)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("delete_timer")
                    .description("Get a detailed description of oh the timer format works")
                    .create_option(|option| {
                        option
                            .name("id")
                            .description("The id of the timer to be deleted")
                            .required(true)
                            .kind(ApplicationCommandOptionType::Number)
                    })
            }).await
            .unwrap();

        GuildId(326330465218985985)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("timer")
                    .description("Create a timer that will create a message in raid_chat with the given options")
                    .create_option(|option| {
                        option
                            .name("title")
                            .description("Title of the message that will be sent out. Must be less than 250 characters. Required")
                            .required(true)
                            .kind(ApplicationCommandOptionType::String)
                    })
                .create_option(|option| {
                    option
                        .name("time")
                        .description("When the timer should send a message. See /timehelp for detailed description of format. Required")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|option| {
                    option
                        .name("channel")
                        .description("The channel to send the message in")
                        .required(true)
                        .kind(ApplicationCommandOptionType::Channel)
                })
                .create_option(|option| {
                    option
                        .name("body")
                        .description("Body of the message that will be sent out. Must be less than 2000 characters. Optional")
                        .required(false)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|option| {
                    option
                        .name("raidlead")
                        .description("Raid leader of the raid. Must be less than 500 characters. Optional")
                        .required(false)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|option| {
                    option
                        .name("dateofmonth")
                        .description("Date of the month must be numerical and between 1 and 31. Optional")
                        .required(false)
                        .kind(ApplicationCommandOptionType::String)
                })
            }).await.unwrap();

        {
            let data_read = &ctx.data.read().await;

            let schedule = data_read
                .get::<Schedule>()
                .expect("Something went wrong gettnig the database connection")
                .lock()
                .await;

            tokio::spawn(schedule.start());
        };

        println!("{} is connected!", ready.user.name);
    }
}

pub async fn delete_timer(command: &Vec<ApplicationCommandInteractionDataOption> ,pool: &PgPool) -> String {
    let mut res = "";

    for option in command {
        if let "id" = option.name.as_str() {
            if let Some(val) = &option.value {
                let parsed_val =  val.to_string().parse::<i32>();
                if let Ok(parsed) = parsed_val {
                    if let Ok(_) = crate::db::delete_timer(pool, parsed).await {
                        res = "No errors while deleting timer";
                    } else {
                        res = "something went wrong while trying to delete the timer"
                    };
                } else {
                    res = "could not parse id, make sure it is an interger";
                }
            }
        } else {
            res = "unknown option";
        };
    };
    res.to_string()
}

pub fn split_to_discord_size(src: String) -> Vec<String> {
    let mut chars = src.chars();
    let sub_string = (0..)
        .map(|_| chars.by_ref().take(1990).collect::<String>())
        .take_while(|s| !s.is_empty())
        .collect::<Vec<_>>();
    sub_string
}
