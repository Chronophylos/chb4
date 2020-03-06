# TODO
- [ ] Add Database
    - [x] User
    - [ ] UserSettings
    - [ ] Person
    - [ ] Ban
    - [x] Channel
    - [ ] ChannelFilter
    - [ ] Copypasta
    - [ ] Voicemail
- [ ] Add Command System
    - [ ] Migrate Commands
    - [ ] Add Command for https://www.babelstone.co.uk/Unicode/whatisit.html
- [ ] Add Action System
    - [ ] Migrate Actions
        - [ ] Add Voicemail Action
- [x] Add description to all actions and commands
- [ ] AFK commands and actions

use https://customapi.aidenwallis.co.uk to make my life easier

## Command System
* Only listen to messages with a prefix
* Invoke Command with arguments from message

## Action System
* Listen to messages matching a regex
* Or to other events (user joined a channel, bot connected)
* Invoke Action with regex match
* every action can be checked at the same time (async)
