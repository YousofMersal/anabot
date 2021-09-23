#![allow(dead_code)]
use scheduler::Job;
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
                ApplicationCommandOptionType, 
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

                                    new_timer.time =val.to_string();
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
                    
                   if let Ok(_) = add_timer(&data, &new_timer).await{
                       {
                           let data_read = &ctx.data.write().await;

                           let schedule = data_read
                               .get::<Schedule>()
                               .expect("Something went wrong gettnig the database connection")
                               .clone();

                           let mut sched_lock = schedule.lock().await;

                           let job = Job::new(&new_timer.time.to_string(), move |_uuid, _l| {
                               channel_raid_warn(&new_timer);
                           });

                           if let Ok(jb) = job {
                               if let Err(e) = sched_lock.add(jb) {
                                   eprintln!("Something went horribly wrong while yelling at people! {}", e);
                               };
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
    If multiple time slots are wanted a comma seperated list is also possible.
To return to last example if for example the same timer needs to be fired twice on multiple days at the same time,
it can be written like so: **19,18 30 Wed,Fri Jun,Jul** which will send a message at 18:30 and 19:30 every wednesday and friday in the months of june and july."
                    .to_owned(),
                "list" => {
                    let mut res = "Listing timers and their id's: \n".to_owned();
                    res = res.to_owned() + "---------\n";
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
                "delete" => {
                    //
                    "test".to_owned()
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
                    .await
                {
                    println!("Cannot respond to slash command: {}", why);
                }
            }

            //&m_split.remove(0);
            //m_split.drain(0..).;
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


      let _res_timers = get_timers(&pool).await;

//    if let Ok(timers) = res_timers {
//        for time in timers {
//            let job = Job::new(&time.time.to_string(), move |_uuid, _l| {
//                //channel_raid_warn(&ctx, &time).await;
//                channel_raid_warn(&time.to_new_timer());
//            });

//            //insert timer job
//        }
//    };

        {
            let data_read = &ctx.data.read().await;

            let schedule = data_read
                .get::<Schedule>()
                .expect("Something went wrong gettnig the database connection")
                .lock()
                .await;

            let t = schedule.start();

            tokio::spawn(async move {
                if let Err(e) =   t.await {
                    eprintln!("Could not start schedule {}", e);
                };
            });
        };

        println!("{} is connected!", ready.user.name);
    }
}

//pub fn process_time(raw_time: &str) -> &str {}

pub fn split_to_discord_size(src: String) -> Vec<String> {
    let mut chars = src.chars();
    let sub_string = (0..)
        .map(|_| chars.by_ref().take(1990).collect::<String>())
        .take_while(|s| !s.is_empty())
        .collect::<Vec<_>>();
    sub_string
}
