table! {
    guilds (id) {
        id -> Int4,
        guild_id -> Int8,
        admin_role -> Nullable<Int8>,
        rules -> Text,
        rules_channel_id -> Nullable<Int8>,
        log_channel_id -> Nullable<Int8>,
        rules_message_id -> Nullable<Int8>,
        reaction_ok -> Varchar,
        reaction_reject -> Varchar,
        active -> Bool,
        strict -> Bool,
        member_role -> Nullable<Int8>,
        preface -> Text,
        postface -> Text,
    }
}

table! {
    rules (id) {
        id -> Int4,
        guild_id -> Int4,
        name -> Varchar,
        rule -> Text,
        extra -> Text,
    }
}

joinable!(rules -> guilds (guild_id));

allow_tables_to_appear_in_same_query!(
    guilds,
    rules,
);
