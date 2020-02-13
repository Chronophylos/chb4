table! {
    bans (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        created -> Datetime,
        until -> Nullable<Datetime>,
    }
}

table! {
    channels (id) {
        id -> Unsigned<Integer>,
        owner_id -> Unsigned<Integer>,
        enabled -> Bool,
        paused -> Bool,
    }
}

table! {
    channel_action_filters (id) {
        id -> Unsigned<Integer>,
        channel_id -> Unsigned<Integer>,
        action_name -> Varchar,
        enable_action -> Bool,
    }
}

table! {
    copypastas (id) {
        id -> Unsigned<Integer>,
        creator_id -> Unsigned<Integer>,
        created -> Datetime,
        name -> Varchar,
        message -> Varchar,
    }
}

table! {
    persons (id) {
        id -> Unsigned<Integer>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        dob -> Nullable<Date>,
    }
}

table! {
    users (id) {
        id -> Unsigned<Integer>,
        twitch_id -> Nullable<Varchar>,
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        first_seen -> Nullable<Datetime>,
        last_seen -> Nullable<Datetime>,
        person_id -> Nullable<Unsigned<Integer>>,
        permission -> Unsigned<Tinyint>,
    }
}

table! {
    user_settings (id) {
        id -> Unsigned<Integer>,
        user_id -> Unsigned<Integer>,
        birthdays -> Bool,
    }
}

table! {
    voicemails (id) {
        id -> Unsigned<Integer>,
        creator_id -> Unsigned<Integer>,
        channel_id -> Unsigned<Integer>,
        receiver_id -> Unsigned<Integer>,
        created -> Datetime,
        scheduled -> Nullable<Datetime>,
        active -> Bool,
        message -> Varchar,
    }
}

joinable!(bans -> users (user_id));
joinable!(channel_action_filters -> channels (channel_id));
joinable!(channels -> users (owner_id));
joinable!(copypastas -> users (creator_id));
joinable!(user_settings -> users (user_id));
joinable!(users -> persons (person_id));
joinable!(voicemails -> channels (channel_id));

allow_tables_to_appear_in_same_query!(
    bans,
    channels,
    channel_action_filters,
    copypastas,
    persons,
    users,
    user_settings,
    voicemails,
);
