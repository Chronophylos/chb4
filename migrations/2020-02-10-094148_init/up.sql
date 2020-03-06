CREATE TABLE people (
    id         SERIAL PRIMARY KEY,
    first_name VARCHAR(45) NULL,
    last_name  VARCHAR(45) NULL,
    dob        DATE NULL
);

CREATE TABLE channels (
    id        SERIAL PRIMARY KEY,
    twitch_id BIGINT NOT NULL,
    enabled   BOOLEAN NOT NULL DEFAULT false,
    paused    BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE channel_action_filters (
    id         SERIAL PRIMARY KEY,
    channel_id INTEGER REFERENCES channels NOT NULL,
    name       VARCHAR(64) NOT NULL,
    enable     BOOLEAN NOT NULL
);

CREATE TABLE channel_command_filters (
    id         SERIAL PRIMARY KEY,
    channel_id INTEGER REFERENCES channels NOT NULL,
    name       VARCHAR(64) NOT NULL,
    enable     BOOLEAN NOT NULL
);

CREATE TABLE user_settings (
    id        SERIAL PRIMARY KEY,
    birthdays BOOLEAN NOT NULL
);

CREATE TABLE users (
    id           SERIAL PRIMARY KEY,
    twitch_id    BIGINT NULL,
    name         VARCHAR(25) NOT NULL,
    display_name VARCHAR(25) NULL,
    first_seen   TIMESTAMP NULL,
    last_seen    TIMESTAMP NULL,
    permission   SMALLINT NOT NULL DEFAULT 0 CHECK (permission >= 0),
    banned_until TIMESTAMP NULL,

    person_id    INTEGER REFERENCES users NULL,
    channel_id   INTEGER REFERENCES channels NULL,
    settings_id  INTEGER REFERENCES user_settings NULL
);

CREATE TABLE copypastas (
    id         SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES users NOT NULL,
    created    TIMESTAMP NOT NULL,
    name       VARCHAR(25) NOT NULL,
    message    VARCHAR(500) NOT NULL
);

CREATE TABLE quotes (
    id         SERIAL PRIMARY KEY,
    creator_id INTEGER REFERENCES users NOT NULL,
    created    TIMESTAMP NOT NULL,
    author     VARCHAR(25) NOT NULL,
    authored   VARCHAR(25) NOT NULL,
    message    VARCHAR(500) NOT NULL
);

CREATE TABLE IF NOT EXISTS voicemails (
    id          SERIAL PRIMARY KEY,
    creator_id  INTEGER REFERENCES users NOT NULL,
    receiver_id INTEGER REFERENCES users NOT NULL,
    created     TIMESTAMP NOT NULL,
    scheduled   TIMESTAMP NULL,
    active      BOOLEAN DEFAULT true,
    message     VARCHAR(500) NOT NULL
);
