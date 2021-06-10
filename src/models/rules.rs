use crate::errors::BotError;
use crate::models::guilds::Guild;
use crate::schema::rules;
use diesel::pg::upsert::*;
use diesel::prelude::*;
use serde::Deserialize;

use diesel::dsl::Eq;
use diesel::dsl::Filter;
use diesel::types::{Int4, Text};

#[derive(Identifiable, Queryable, Associations, Debug)]
#[belongs_to(Guild)]
pub struct Rule {
    pub id: i32,
    pub guild_id: i32,
    pub name: String,
    pub rule: String,
    pub extra: String,
}

impl Rule {
    pub fn format_short(&self) -> String {
        format!("**{}**\n{}", self.name, self.rule)
    }

    pub fn format_long(&self) -> String {
        format!("**{}**\n{}\n\n{}", self.name, self.rule, self.extra)
    }

    pub fn drop(&self, connection: &PgConnection) -> Result<usize, BotError> {
        use crate::schema::rules::dsl::*;
        Ok(diesel::delete(rules.filter(id.eq(self.id))).execute(connection)?)
    }

    pub fn drop_rule(
        name: &str,
        guild_id: i32,
        connection: &PgConnection,
    ) -> Result<usize, BotError> {
        Ok(diesel::delete(
            rules::table.filter(rules::name.eq(name).and(rules::guild_id.eq(guild_id))),
        )
        .execute(connection)?)
    }
}

#[derive(Insertable, Debug, Deserialize, AsChangeset)]
#[table_name = "rules"]
pub struct NewRule {
    pub guild_id: i32,
    pub name: String,
    pub rule: String,
    pub extra: String,
}
impl NewRule {
    pub fn upsert(self, connection: &PgConnection) -> Result<Rule, String> {
        use crate::schema::rules::dsl::*;
        let a = diesel::insert_into(rules)
            .values(&self)
            .on_conflict(on_constraint("uc_rule_by_guild"))
            .do_update()
            //.set((rule.eq(excluded(rule)), extra.eq(excluded(extra)))
            .set(&self)
            .get_result::<Rule>(connection);

        log::error!("aaa\n{:?}", a,);
        a.map_err(|_| "could not insert new rule".to_string())
    }
}
