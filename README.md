# discord rules

Simple bot to give users on a discord channel a certain role depending if they
reacted an acceptance emoji to a certain message.
Mainly supposed to give users access to protected channels if they accept a
rules message.

## To run:
```
go get github.com/obayemi/rules-bot
rules -t <discord token> -d <path to sqlite db>
```

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
- `set-message-id`: set the ID of the message the bot will track to give / take permission

- `enable`: put the rules message in the rules channel, start tracking reactions and assigning role
- `disable`: stop tracking the reactions to the rules message
- `update`: apply recent rules change to the 
- `status`: show configuration

## TODO
- ~read reactions at initialization to allow people in that accepted the rules when the bot was offline (strict mode)~ -> can't get more than 100 reactions to a message in discord
- ~strict mode togle, to not enforce every user to leave a like on the rules comment. (like the recheck at initialisation)~
- improve commands interface
- unit tests
- keep track of what happens on the server: roles deleted, channels deleted, rules message deleted
- dockerfile for even easier/secure deployment ?
