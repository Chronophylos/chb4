CREATE TABLE persons (
    id         INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    first_name VARCHAR(45) NULL,
    last_name  VARCHAR(45) NULL,
    dob        DATE NULL
);

CREATE TABLE users (
    id           INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    twitch_id    VARCHAR(16) NULL,
    name         VARCHAR(25) NOT NULL,
    display_name VARCHAR(25) NULL,
    first_seen   DATETIME NULL,
    last_seen    DATETIME NULL,
    person_id    INT UNSIGNED NULL,
    permission   TINYINT UNSIGNED NOT NULL DEFAULT 0,

    CONSTRAINT fk_users_persons
        FOREIGN KEY (person_id)
        REFERENCES persons (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE user_settings (
    id        INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id   INT UNSIGNED NOT NULL,
    birthdays BOOLEAN NOT NULL,

    CONSTRAINT fk_user_settings_users
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE bans (
    id      INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    user_id INT UNSIGNED NOT NULL,
    created DATETIME NOT NULL,
    until   DATETIME NULL,

    CONSTRAINT fk_ban_user
        FOREIGN KEY (user_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE copypastas (
    id         INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    creator_id INT UNSIGNED NOT NULL,
    created    DATETIME NOT NULL,
    name       VARCHAR(25) NOT NULL,
    message    VARCHAR(500) NOT NULL,

    CONSTRAINT fk_copypastas_users
        FOREIGN KEY (creator_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE channels (
    id       INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    owner_id INT UNSIGNED NOT NULL,
    enabled  BOOLEAN NOT NULL DEFAULT false,
    paused   BOOLEAN NOT NULL DEFAULT false,

    CONSTRAINT fk_channel_user
        FOREIGN KEY (owner_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE channel_action_filters (
    id            INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    channel_id    INT UNSIGNED NOT NULL,
    action_name   VARCHAR(64) NOT NULL,
    enable_action BOOLEAN NOT NULL COMMENT 'Should this action be activated or deactivated?',

    CONSTRAINT fk_action_filter_channel
        FOREIGN KEY (channel_id)
        REFERENCES channels (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);

CREATE TABLE IF NOT EXISTS voicemails (
    id          INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    creator_id  INT UNSIGNED NOT NULL,
    channel_id  INT UNSIGNED NOT NULL,
    receiver_id INT UNSIGNED NOT NULL,
    created     DATETIME NOT NULL,
    scheduled   DATETIME NULL,
    active      BOOLEAN NOT NULL DEFAULT true,
    message     VARCHAR(500) NOT NULL,

    CONSTRAINT fk_voicemails_creator
        FOREIGN KEY (creator_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT,

    CONSTRAINT fk_voicemails_channels
        FOREIGN KEY (channel_id)
        REFERENCES channels (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT,

    CONSTRAINT fk_voicemails_receiver
        FOREIGN KEY (receiver_id)
        REFERENCES users (id)
        ON DELETE CASCADE
        ON UPDATE RESTRICT
);
