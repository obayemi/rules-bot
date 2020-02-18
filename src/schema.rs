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
    }
}
