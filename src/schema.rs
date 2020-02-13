table! {
    guilds (id) {
        id -> Int4,
        guild_id -> Varchar,
        admin_role -> Nullable<Varchar>,
        rules -> Text,
        rules_channel_id -> Nullable<Varchar>,
        log_channel_id -> Nullable<Varchar>,
        rules_message_id -> Nullable<Varchar>,
        reaction_ok -> Varchar,
        reaction_reject -> Varchar,
        active -> Bool,
        strict -> Bool,
    }
}
