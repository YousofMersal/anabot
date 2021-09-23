#![allow(dead_code)]
use std::{env, sync::Arc};

use scheduler::*;
use serenity::{futures::lock::Mutex, prelude::TypeMapKey};
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    types::Decimal,
    Error, PgPool,
};

enum Time {
    Timer(Timer),
    DbTimer(DbTimer),
    NewTimer(NewTimer),
}

#[derive(Default)]
pub struct Timer {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: u64,
}

pub struct DbTimer {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: Decimal,
}

#[derive(Default)]
pub struct NewTimer {
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: u64,
}

impl Timer {
    pub fn to_new_timer(self) -> NewTimer {
        NewTimer {
            title: self.title,
            time: self.time,
            body: self.body,
            recurring: self.recurring,
            raid_lead: self.raid_lead,
            channel: self.channel,
        }
    }
}

//Serinity way to add data to the global bot context
pub struct DB;
pub struct Schedule;

impl TypeMapKey for DB {
    type Value = Arc<PgPool>;
}

impl TypeMapKey for Schedule {
    type Value = Arc<Mutex<JobScheduler>>;
}

pub async fn establish_db_connection() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mypool = PgPool::connect(&database_url)
        .await
        .expect("Could not create connection pool");

    mypool
}

pub async fn add_timer(pool: &PgPool, timer: &NewTimer) -> Result<Vec<PgRow>, Error> {
    let res = query_as!(
        Timer,
        r#"
INSERT INTO timers (title, body, recurring, raid_lead, time, channel)
Values ($1, $2, $3, $4, $5, $6)"#,
        timer.title,
        timer.body,
        timer.recurring,
        timer.raid_lead,
        timer.time,
        Decimal::from(timer.channel),
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub async fn delete_timer(pool: &PgPool, id: i32) -> Result<PgQueryResult, Error> {
    let res = query!(
        "DELETE FROM timers
            WHERE id = $1",
        id
    )
    .execute(pool)
    .await?;

    Ok(res)
}

pub async fn get_timers(pool: &PgPool) -> Result<Vec<Timer>, Error> {
    let res = query_as!(
        DbTimer,
        "SELECT *
        FROM timers"
    )
    .map(|dbt| {
        let string = dbt.channel.to_string();
        let num = string.parse::<u64>().unwrap();
        Timer {
            id: dbt.id,
            title: dbt.title,
            body: dbt.body,
            recurring: dbt.recurring,
            raid_lead: dbt.raid_lead,
            time: dbt.time,
            channel: num,
        }
    })
    .fetch_all(pool)
    .await?;

    Ok(res)
}

pub fn convert_string(input: &String) {
    let mut res: Result<String, std::io::Error> = Ok(input.to_string());
    let split_itt = input.split(" ");
    let split: Vec<&str> = split_itt.collect();
    //Check amount of arguments
    if split.len() > 6 || split.len() < 3 {
        //Check hour
        let hour_split = split[0].split(",");
        for _hour_u in hour_split {
            let hour_s = split[0].parse::<i64>();
            if let Ok(hour) = hour_s {
                if hour > 24 || hour < 0 {
                    res = Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid format",
                    ));
                }
            }
        }

        let min_split = split[1].split(",");
        for min_u in min_split {
            let min_s = min_u.parse::<i64>();

            if let Ok(min) = min_s {
                if min > 60 || min < 0 {
                    res = Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "invalid format",
                    ));
                }
            } else {
                res = Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid format",
                ));
            }
        }
    }
    let _new_string = String::new();
    for _elem in split {
        //
    }
    if let Err(e) = res {
        eprintln!("{}", e);
    };
}
