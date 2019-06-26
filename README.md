# discord rules

Simple bot to give users on a discord channel a certain role depending if they
reacted an acceptance emoji to a certain message.
Mainly supposed to give users access to protected channels if they accept a
rules message.

## To run:
> go get github.com/obayemi/discord-rules
> rules -t <discord token> -d <path to sqlite db>

If no db file is given, will run with in-memory non-persistent DB

## To invoke:
mention to run commands, mention should be first token of the message.
only guild owner or member of the admin group are allowed to run commands (TODO)

## Commands:
- `set-rules`: set the rules, put them inside a block code (markdown allowed)
- `set-rules-channel`: set the channel the bot will post its message in
- `set-logs-channel`: set the channel the bot will log its interractions with users in
- `set-reactions`: set the emojis used for yes/no answers if you want to change them
- `set-role`: set the role the bot will give to your users after they accept the rules
- `set-admin-role`: set the role that will be allowed to interract with the bot (admins/moderators)

- `enable`: put the rules message in the rules channel, start tracking reactions and assigning role
- `disable`: stop tracking the reactions to the rules message
- `update`: apply recent rules change to the 
- `status`: show configuration

## TODO
- actualy detect that the ok/nok reactions were sent and give appropriate role
- auto update rules if enabled
- roles management: set-role, set-admin-role
- read reactions at initialization to allow people in that accepted the rules when the bot was offline
- improve commands interface
- command to start the bot by force tracking a message not initially sent by the bot
- unit tests
