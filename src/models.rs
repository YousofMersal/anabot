#[derive(Queryable)]
pub struct Timer {
    pub id: i32,
    pub title: String,
    pub body: Option<String>,
    pub recurring: bool,
    pub raid_lead: Option<String>,
    pub time: String,
}

use super::schema::timers;

#[derive(Insertable)]
#[table_name = "timers"]
pub struct NewTimer<'a> {
    pub title: &'a str,
    pub body: Option<&'a str>,
    pub recurring: bool,
    pub raid_lead: Option<&'a str>,
    pub time: &'a str,
}
