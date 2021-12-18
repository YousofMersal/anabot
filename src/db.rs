use std::{env, fmt, str::FromStr, sync::Arc};

use chrono::{DateTime, Local};
use serenity::{futures::lock::Mutex, prelude::TypeMapKey};
use sqlx::{query_as, types::Decimal, Error, PgPool};
use tokio_cron_scheduler::*;
use uuid::Uuid;

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
    All,
}

impl fmt::Display for WeekDay {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl FromStr for WeekDay {
    type Err = WeekError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let sl: &str = &s.to_lowercase();
        match sl {
            "mon" => Ok(WeekDay::Mon),
            "tue" => Ok(WeekDay::Tue),
            "wed" => Ok(WeekDay::Wed),
            "thu" => Ok(WeekDay::Thu),
            "fri" => Ok(WeekDay::Fri),
            "sat" => Ok(WeekDay::Sat),
            "sun" => Ok(WeekDay::Sun),
            "*" => Ok(WeekDay::All),
            _ => Err(WeekError::new("Invalid WeekDay")),
        }
    }
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
    pub uuid: Uuid,
}

pub struct DbTimer {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: Decimal,
    pub uuid: Uuid,
}

#[derive(Default, Clone)]
pub struct NewTimer {
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
    pub channel: u64,
    pub uuid: Uuid,
}

impl Timer {
    pub fn _to_new_timer(&self) -> NewTimer {
        NewTimer {
            title: self.title.clone(),
            time: self.time.clone(),
            body: self.body.clone(),
            recurring: self.recurring,
            raid_lead: self.raid_lead.clone(),
            channel: self.channel,
            uuid: self.uuid,
        }
    }

    /// Get the next upcoming fire time in a human readable format
    /// Ex: 2021-12-18 18:06:00
    pub fn get_human_time(&self) -> String {
        let crn = cron::Schedule::from_str(&self.time);
        if let Ok(val) = crn {
            if let Some(date) = val.upcoming(chrono::Utc).next() {
                let c: DateTime<Local> = DateTime::from(date);
                c.naive_local().to_string()
            } else {
                "could not convert time".to_owned()
            }
        } else {
            "could not convert time".to_owned()
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

//TODO: Make docs
pub async fn establish_db_connection() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mypool = PgPool::connect(&database_url)
        .await
        .expect("Could not create connection pool");

    mypool
}

//TODO: Make docs
pub async fn add_timer(pool: &PgPool, timer: &NewTimer) -> Result<i32, Error> {
    let res = query_as!(
        DbTimer,
        r#"
INSERT INTO timers (title, body, recurring, raid_lead, time, channel, uuid)
Values ($1, $2, $3, $4, $5, $6, $7)
RETURNING *"#,
        timer.title,
        timer.body,
        timer.recurring,
        timer.raid_lead,
        timer.time,
        Decimal::from(timer.channel),
        timer.uuid
    )
    .map(|t| t.id)
    .fetch_one(pool)
    .await?;

    Ok(res)
}

//TODO: Make docs
pub async fn db_delete_timer(pool: &PgPool, id: i32) -> Result<Option<i64>, Error> {
    let res = query!(
        "WITH deleted AS (
        DELETE FROM timers
            WHERE id = $1
            RETURNING *)
         SELECT count(*) FROM deleted",
        id
    )
    .map(|num| num.count)
    .fetch_one(pool)
    .await?;

    Ok(res)
}

pub async fn get_uuid(id: i32, pool: &PgPool) -> Result<Uuid, Error> {
    let res = query!(
        r#"SELECT uuid
        FROM timers
        WHERE id=$1"#,
        id
    )
    .map(|u| u.uuid)
    .fetch_one(pool)
    .await?;

    Ok(res)
}

//TODO: Make docs
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
            uuid: dbt.uuid,
        }
    })
    .fetch_all(pool)
    .await?;

    Ok(res)
}

/// Naively put the string recived into a cron time format.
/// Anything given is assumed to be correct.
pub fn naive_convert(input: &str) -> Result<String, &str> {
    let split_itt = input.split(" ");
    let split: Vec<&str> = split_itt.collect();
    let local_to_utc_hour = &split[0].trim_matches('"').parse::<i32>().unwrap()
        - (Local::now().offset().local_minus_utc() / 3600);
    let res;
    if split.len() == 3 {
        res = format!(
            "0 {} {} * * {}",
            split[1].trim_matches('"'),
            local_to_utc_hour.to_string(),
            split[2].trim_matches('"')
        );
    } else if split.len() == 4 {
        res = format!(
            "0 {} {} * {} {}",
            split[1].trim_matches('"'),
            local_to_utc_hour.to_string(),
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
pub fn is_valid_cron(input: &str) -> Result<String, String> {
    let res = input.to_string();

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
                    return Err(
                        "Invalid format, reason: Hour must be 24 hour format and between 0 and 23"
                            .to_string(),
                    );
                }
            }
        }

        // Check min
        let min_split = split[1].split(',');
        for min_u in min_split {
            let min_s = min_u.parse::<i32>();
            if let Ok(min) = min_s {
                if min > 59 || min < 0 {
                    return Err(
                        "Invalif format, reason: Minutes must be between 0 and 59.".to_string()
                    );
                }
            }
        }

        //Check day
        let day_split = split[2].split(',');
        for day_u in day_split {
            let day_u = day_u.trim_matches('"');
            if let Err(week_err) = WeekDay::from_str(day_u) {
                return Err(week_err.details.to_string());
            }
        }

        if split.len() == 4 {
            let month_split = split[3].split(',');
            for month_u in month_split {
                let month_s = month_u.parse::<i32>();
                if let Ok(month) = month_s {
                    if month > 1 || month < 12 {
                        return Err(
                            "Invalif format, reason: Month must be between 1 and 12.".to_string()
                        );
                    }
                }
            }
        }
    } else {
        return Err(
            "could not convert string, double check that the format is correct.".to_string(),
        );
    }
    Ok(res)
}
