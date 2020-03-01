table! {
    channels (id) {
        id -> Unsigned<Integer>,
        twitch_id -> Unsigned<Bigint>,
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
    channel_command_filters (id) {
        id -> Unsigned<Integer>,
        channel_id -> Unsigned<Integer>,
        command_name -> Varchar,
        enable_command -> Bool,
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
    people (id) {
        id -> Unsigned<Integer>,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        dob -> Nullable<Date>,
    }
}

table! {
    users (id) {
        id -> Unsigned<Integer>,
        twitch_id -> Nullable<Unsigned<Bigint>>,
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        first_seen -> Nullable<Datetime>,
        last_seen -> Nullable<Datetime>,
        permission -> Unsigned<Tinyint>,
        banned_until -> Nullable<Datetime>,
        person_id -> Nullable<Unsigned<Integer>>,
        channel_id -> Nullable<Unsigned<Integer>>,
        settings_id -> Nullable<Unsigned<Integer>>,
    }
}

table! {
    user_settings (id) {
        id -> Unsigned<Integer>,
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

joinable!(channel_action_filters -> channels (channel_id));
joinable!(channel_command_filters -> channels (channel_id));
joinable!(copypastas -> users (creator_id));
joinable!(users -> channels (channel_id));
joinable!(users -> people (person_id));
joinable!(users -> user_settings (settings_id));
joinable!(voicemails -> channels (channel_id));

allow_tables_to_appear_in_same_query!(
    channels,
    channel_action_filters,
    channel_command_filters,
    copypastas,
    people,
    users,
    user_settings,
    voicemails,
);
