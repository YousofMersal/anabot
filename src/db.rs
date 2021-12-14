#![allow(dead_code)]
use std::{env, fmt, str::FromStr, sync::Arc};

use scheduler::*;
use serenity::{futures::lock::Mutex, prelude::TypeMapKey};
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    types::Decimal,
    Error, PgPool,
};

#[derive(Debug)]
struct WeekError {
    details: String,
}

impl WeekError {
    fn new(msg: &str) -> WeekError {
        WeekError {
            details: msg.to_string(),
        }
    }
}

impl std::fmt::Display for WeekError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for WeekError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[derive(Debug, PartialEq)]
enum WeekDay {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl WeekDay {
    fn to_string(s: &Self) -> &str {
        match s {
            WeekDay::Mon => "Mon",
            WeekDay::Tue => "Tue",
            WeekDay::Wed => "Wed",
            WeekDay::Thu => "Thu",
            WeekDay::Fri => "Fri",
            WeekDay::Sat => "Sat",
            WeekDay::Sun => "Sun",
        }
    }
}

impl FromStr for WeekDay {
    type Err = WeekError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Mon" => Ok(WeekDay::Mon),
            "Tue" => Ok(WeekDay::Tue),
            "Wed" => Ok(WeekDay::Wed),
            "Thu" => Ok(WeekDay::Thu),
            "Fri" => Ok(WeekDay::Fri),
            "Sat" => Ok(WeekDay::Sat),
            "Sun" => Ok(WeekDay::Sun),
            _ => Err(WeekError::new("Invalid WeekDay")),
        }
    }
}

enum Time {
    Timer(Timer),
    DbTimer(DbTimer),
    NewTimer(NewTimer),
}

#[derive(Default, Clone)]
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

#[derive(Default, Clone)]
pub struct NewTimer {
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: u64,
}

impl Timer {
    pub fn to_new_timer(&self) -> NewTimer {
        NewTimer {
            title: self.title.clone(),
            time: self.time.clone(),
            body: self.body.clone(),
            recurring: self.recurring,
            raid_lead: self.raid_lead.clone(),
            channel: self.channel,
        }
    }
}

impl fmt::Display for Timer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Title: {}\nTime: {}", self.title, self.time)
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

pub fn naive_convert(input: &str) -> Result<String, &str> {
    let split_itt = input.split(" ");
    let split: Vec<&str> = split_itt.collect();
    let res;
    if split.len() == 3 {
        res = format!(
            "0 {} {} * * {}",
            split[1].trim_matches('"'),
            split[0].trim_matches('"'),
            split[2].trim_matches('"')
        );
    } else if split.len() == 4 {
        res = format!(
            "0 {} {} * {} {}",
            split[1].trim_matches('"'),
            split[0].trim_matches('"'),
            split[2].trim_matches('"'),
            split[3].trim_matches('"')
        );
    } else {
        return Err("could not format string");
    }
    Ok(res)
}

/// Convert to valid cron string.
/// If string is not already a valid cron string construct of partial artifacts.
/// If not able to construct cron string will return error.
///
/// ```rust
/// let cron-t = anabot::convert_string("12 00 Thu");
/// assert_eq!(cron-t, "* 12 00 * * Thu *");
/// ```
pub fn convert_string(input: &str) -> Result<&str, Box<dyn std::error::Error>> {
    let mut res: Result<&str, Box<dyn std::error::Error>> = Ok(input);

    let split_itt = input.split(" ");
    let split: Vec<&str> = split_itt.collect();
    //Check amount of arguments
    if split.len() == 3 {
        //Check hour
        let hour_split = split[0].split(',');
        for hour_u in hour_split {
            let hour_s = hour_u.parse::<i32>();
            if let Ok(hour) = hour_s {
                if hour > 23 || hour < 0 {
                    res = Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalid format, reason: Hour must be 24 hour format and between 0 and 23",
                    )));
                }
            }
        }

        // Check min
        let min_split = split[1].split(',');
        for min_u in min_split {
            let min_s = min_u.parse::<i32>();
            if let Ok(min) = min_s {
                if min > 59 || min < 0 {
                    res = Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalif format, reason: Minutes must be between 0 and 59.",
                    )));
                }
            }
        }

        //Check day
        let day_split = split[2].split(',');
        for day_u in day_split {
            if let Err(e) = WeekDay::from_str(day_u) {
                res = Err(Box::new(e));
            }
        }
    } else if split.len() == 4 {
        let month_split = split[3].split(',');
        for month_u in month_split {
            let month_s = month_u.parse::<i32>();
            if let Ok(month) = month_s {
                if month > 1 || month < 12 {
                    res = Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Invalif format, reason: Month must be between 1 and 12.",
                    )));
                }
            }
        }
    } else {
        res = Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "could not convert string",
        )));
    }

    res
}
