use crate::schema::*;

#[derive(Queryable, Identifiable)]
#[table_name = "channels"]
pub struct Channel {
    pub id: i32,
    pub twitch_id: i64,
    pub enabled: bool,
    pub paused: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Channel)]
#[table_name = "channel_action_filters"]
pub struct ChannelActionFilter {
    pub id: i32,
    pub channel_id: i32,
    pub action_name: String,
    pub enable_action: bool,
}

#[derive(Queryable, Identifiable, Associations)]
#[belongs_to(Channel)]
#[table_name = "channel_command_filters"]
pub struct ChannelCommandFilter {
    pub id: i32,
    pub channel_id: i32,
    pub command_name: String,
    pub enable_command: bool,
}
