use crate::models::guilds::Guild;
use crate::schema::rules;
use serde::Deserialize;
//use diesel::prelude::*;

#[derive(Identifiable, Queryable, Associations, Debug)]
#[belongs_to(Guild)]
pub struct Rule {
    pub id: i32,
    pub guild_id: i32,
    pub name: String,
    pub rule: String,
    pub extra: String,
}

#[derive(Insertable, Debug, Deserialize)]
#[table_name = "rules"]
pub struct NewRule {
    pub guild_id: i32,
    pub name: String,
    pub rule: String,
    pub extra: String,
}
