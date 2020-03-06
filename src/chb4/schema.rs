table! {
    channel_action_filters (id) {
        id -> Int4,
        channel_id -> Int4,
        name -> Varchar,
        enable -> Bool,
    }
}

table! {
    channel_command_filters (id) {
        id -> Int4,
        channel_id -> Int4,
        name -> Varchar,
        enable -> Bool,
    }
}

table! {
    channels (id) {
        id -> Int4,
        twitch_id -> Nullable<Int8>,
        enabled -> Bool,
        paused -> Bool,
    }
}

table! {
    copypastas (id) {
        id -> Int4,
        creator_id -> Int4,
        created -> Timestamp,
        name -> Varchar,
        message -> Varchar,
    }
}

table! {
    people (id) {
        id -> Int4,
        first_name -> Nullable<Varchar>,
        last_name -> Nullable<Varchar>,
        dob -> Nullable<Date>,
    }
}

table! {
    quotes (id) {
        id -> Int4,
        creator_id -> Int4,
        created -> Timestamp,
        author -> Varchar,
        authored -> Varchar,
        message -> Varchar,
    }
}

table! {
    user_settings (id) {
        id -> Int4,
        birthdays -> Bool,
    }
}

table! {
    users (id) {
        id -> Int4,
        twitch_id -> Nullable<Int8>,
        name -> Varchar,
        display_name -> Nullable<Varchar>,
        first_seen -> Nullable<Timestamp>,
        last_seen -> Nullable<Timestamp>,
        permission -> Int2,
        banned_until -> Nullable<Timestamp>,
        person_id -> Nullable<Int4>,
        channel_id -> Nullable<Int4>,
        settings_id -> Nullable<Int4>,
    }
}

table! {
    voicemails (id) {
        id -> Int4,
        creator_id -> Int4,
        receiver_id -> Int4,
        created -> Timestamp,
        scheduled -> Nullable<Timestamp>,
        active -> Nullable<Bool>,
        message -> Varchar,
    }
}

joinable!(channel_action_filters -> channels (channel_id));
joinable!(channel_command_filters -> channels (channel_id));
joinable!(copypastas -> users (creator_id));
joinable!(quotes -> users (creator_id));
joinable!(users -> channels (channel_id));
joinable!(users -> user_settings (settings_id));

allow_tables_to_appear_in_same_query!(
    channel_action_filters,
    channel_command_filters,
    channels,
    copypastas,
    people,
    quotes,
    user_settings,
    users,
    voicemails,
);
