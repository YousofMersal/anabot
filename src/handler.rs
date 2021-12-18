use scheduler::Job;
use serenity::model::id::GuildId;
use sqlx::PgPool;
use crate::db::*;

use crate::channel_raid_warn;

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        interactions::{
            application_command::{
                 ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType 
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
                    let mut res = String::new();
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
                                    if let Ok(time) = is_valid_cron(&str) {
                                        let conversion = naive_convert(&time);
                                        if let Ok(value) = conversion {
                                            new_timer.time = value;
                                        } else {
                                            res = "Time format malformed, for help use /timerhelp".to_string();
                                        }
                                    } else {
                                        res = "Time format malformed, for help use /timerhelp".to_string();
                                    }
                                }
                            }
                            "channel" => {
                                if let Some(opt) = &option.resolved {
                                    match opt {
                                        ApplicationCommandInteractionDataOptionValue::Channel(chan) => {
                                            if let serenity::model::channel::ChannelType::Text = chan.kind {
                                                new_timer.channel = *chan.id.as_u64();
                                            } else {
                                                res = "Please select a normal text channel when selecting channel to announce in.".to_string();
                                            }
                                        },
                                        _ => eprintln!("unknown option found while createing new timer")

                                    }
                                }
                            }
                            _ => eprintln!("unknown option found while createing new timer")
                        }
                    }

                    let clock = &mut new_timer.clone();
                    let mut job_added = false;

                    {
                        let data_read = &ctx.data.write().await;

                        let schedule = data_read
                            .get::<Schedule>()
                            .expect("Something went wrong getting the database connection");

                        let mut sched_lock = schedule.lock().await;

                        let job = Job::new(&new_timer.time.clone(), move |_uuid, _l| {
                            channel_raid_warn(new_timer.clone());
                        });

                        if let Ok(j) = job {
                            let guid = j.guid().clone();
                            clock.uuid = guid;
                            sched_lock.add(j).expect("error adding job to queue"); 
                            job_added = true;
                            res = "Timer succesfully registered".to_string();
                        }
                    }

                    if job_added {
                        if let Err(_) = add_timer(&data, clock).await {
                            res = "could not add the timer to the database".to_string()
                        } 
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
                                        //res = res + "recurring: " + &timer.recurring.to_string() + "\n";
                                        res = res + "Next time of fire: " + &timer.get_human_time() + "\n";
                                        res = res + "Set timer: " + &timer.time.to_string() + "\n";
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
                    // TODO:fix this so when everything is correct it responds correctly
                    let command_options = &command.data.options; 
                    let mut res = String::new();
                    let mut queue_removed = false;
                    if let Some(id) = &command_options[0].value {
                        if let Ok(parsed_val) = id.to_string().parse::<i32>(){
                            {
                                let data_read = &ctx.data.read().await;

                                let mut schedule = data_read
                                    .get::<Schedule>()
                                    .expect("Something went wrong gettnig the database connection")
                                    .lock()
                                    .await;

                                let timer_uuid = get_uuid(parsed_val, &data).await;

                                if let Ok(is_uuid) = timer_uuid {
                                    if let Err(removal_erro) = schedule.remove(&is_uuid) {
                                        eprintln!("remove from sched error: {}", removal_erro);
                                        res = String::from("Something went wrong while removing timer");
                                    } else {
                                        queue_removed = true;
                                    }
                                }
                            };
                            if queue_removed {
                                res = delete_timer(parsed_val, &data).await;
                            };
                        } else {
                            res = "could not parse ID, make sure it's a positive whole number".to_string();
                        }
                    } else {
                        res = "could not get id from command please try again".to_string();
                    }
                    res
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

/// Takes and ID and a refference to a "pool" and, delets a row with the given ID
pub async fn delete_timer(val: i32 ,pool: &PgPool) -> String {
    let mut res: String = String::new();

    if let Ok(num) = db_delete_timer(pool, val).await {
        if let Some(rows_deleted) = num {
            res = format!("Deleted {} timers", rows_deleted)
        }
    } else {
        res = "something went wrong while trying to delete the timer".to_string();
    };

    res.to_string()
}

/// Splits string into parts that Discord can digest.
pub fn split_to_discord_size(src: String) -> Vec<String> {
    let mut chars = src.chars();
    let sub_string = (0..)
        .map(|_| chars.by_ref().take(1990).collect::<String>())
        .take_while(|s| !s.is_empty())
        .collect::<Vec<_>>();
    sub_string
}
