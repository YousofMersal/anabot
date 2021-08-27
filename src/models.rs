use std::{env, sync::Arc};

use scheduler::*;
use serenity::prelude::TypeMapKey;
use sqlx::{
    postgres::{PgQueryResult, PgRow},
    Error, PgPool,
};

pub struct Timer {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
}

//Serinity way to add data to the global bot context
pub struct DB;
pub struct Schedule;

impl TypeMapKey for DB {
    type Value = Arc<PgPool>;
}

impl TypeMapKey for Schedule {
    type Value = Arc<JobScheduler>;
}

pub async fn establish_db_connection() -> PgPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mypool = PgPool::connect(&database_url)
        .await
        .expect("Could not create connection pool");

    mypool
}

async fn add_timer(
    pool: &PgPool,
    title: String,
    body: Option<String>,
    recurring: bool,
    raid_lead: Option<String>,
    time: String,
) -> Result<Vec<PgRow>, Error> {
    let res = query_as!(
        Timer,
        r#"
INSERT INTO timers (title, body, recurring, raid_lead, time)
Values ($1, $2, $3, $4, $5)"#,
        title,
        body,
        recurring,
        raid_lead,
        time
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
        Timer,
        "SELECT *
        FROM timers"
    )
    .fetch_all(pool)
    .await?;

    Ok(res)
}
