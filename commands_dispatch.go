package main

import (
	"fmt"
	"log"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

var registry = CommandsRegistry{}

func init() {
	registry.register(setRulesContent, "set-rules", "`\u200B`\u200B`<RULES>`\u200B`\u200B`", "set the rules, put them inside a block code (markdown allowed)")
	registry.register(setRulesChannel, "set-rules-channel", "#<CHANNEL>", "set the channel the bot will post its message in")
	registry.register(setLogsChannel, "set-logs-channel", "#<CHANNEL>", "set the channel the bot will log its interractions with users in")
	registry.register(setReactions, "set-reactions", "<OK> <KO>", "set the emojis used for yes/no answers if you want to change them")
	registry.register(setRole, "set-role", "@<ROLE>", "set the role the bot will give to your users after they accept the rules")
	registry.register(setAdminRole, "set-admin-role", "@<ROLE>", "set the role that will be allowed to interract with the bot (admins/moderators)")
	registry.register(setRuleMessageID, "set-message-id", "<MESSAGE_ID>", "set the ID of the message the bot will track to give / take permission")

	registry.register(enableRules, "enable", "", "put the rules message in the rules channel, start tracking reactions and assigning role")
	registry.register(disableRules, "disable", "", "stop tracking the reactions to the rules message")
	registry.register(updateRules, "update", "", "apply recent rules change to the ")
	registry.register(showStatus, "status", "", "show configuration")
}

type TriggerFunc func(*Server, *discordgo.Message, []string)
type Trigger struct {
	Trigger    string
	Command    TriggerFunc
	Invocation string
	Help       string
}
type CommandsRegistry struct {
	Commands []Trigger
}

func (r *CommandsRegistry) register(command TriggerFunc, trigger string, invocation string, help string) {
	r.Commands = append(r.Commands, Trigger{trigger, command, invocation, help})
}

func botTriggered(botID string, m *discordgo.Message) bool {
	for _, user := range m.Mentions {
		if user.ID == botID {
			return true
		}
	}
	return false
}
func authorizedUser(s *discordgo.Session, guild *discordgo.Guild, authorID string, server *Server) bool {
	if authorID == guild.OwnerID {
		return true
	}
	member, err := s.GuildMember(guild.ID, authorID)
	if err != nil {
		log.Println("request by invalid user")
		return false
	}
	for _, role := range member.Roles {
		log.Println(role)
		if role == server.AdminRole {
			return true
		}
	}
	return false
}

func messageHandler(s *discordgo.Session, m *discordgo.MessageCreate) {
	if m.Author.ID == s.State.User.ID || !botTriggered(s.State.User.ID, m.Message) {
		return
	}
	guild, err := s.Guild(m.GuildID)
	if err != nil {
		log.Printf("command on invalid server %s: %s", m.GuildID, m.Content)
		return
	}
	server := Server{}
	result := db.Where(Server{GuildID: m.GuildID}).FirstOrInit(&server)

	if !authorizedUser(s, guild, m.Author.ID, &server) {
		// TODO: check ig author is part of admin role
		return
	}

	// let the menthion be the rirst thing in the comment
	reg := regexp.MustCompile(fmt.Sprintf("<@!?(%s)>", s.State.User.ID))
	if reg.FindStringIndex(m.Content)[0] != 0 {
		return
	}

	log.Printf("%s: %s\n", m.Author.ID, m.Content)

	/*
	   result.RecordNotFound() is not useful now, but I keep it in place to ease the eventual migration from `FirstOrInit` to `First`
	*/
	if result.RecordNotFound() || db.NewRecord(server) {
		log.Printf("command on unregistered server %s: %s", m.GuildID, m.Content)
	} else if result.Error != nil {
		log.Println(result.Error)
		return
	}
	fields := strings.Fields(m.Content)

	// no point to continue if there's no command. Help maybe?
	if len(fields) <= 1 {
		return
	}
	for _, command := range registry.Commands {
		if command.Trigger == fields[1] {
			command.Command(&server, m.Message, fields[2:])
			return
		}
	}
	showHelp(m.ChannelID)
}

func showHelp(channelID string) {
	help := []string{}
	for _, command := range registry.Commands {
		help = append(help, fmt.Sprintf("`%s %s`: %s", command.Trigger, command.Invocation, command.Help))
	}
	DiscordSession.ChannelMessageSend(channelID, "```\n"+strings.Join(help, "\n")+"```")
}
