package main

import (
	"errors"
	"fmt"
	"regexp"

	"github.com/bwmarrin/discordgo"
)

/*
 * configure the bot (rules, channels for rules and logs, change reactions, etc...
 */
func setRulesContent(server *Server, m *discordgo.Message, fields []string) {
	re := regexp.MustCompile("(?s)```\n?(.+)\n?```")
	results := re.FindSubmatch([]byte(m.Content))
	if len(results) != 2 {
		DiscordSession.ChannelMessageSend(m.ChannelID, "malformed rules")
		return
	}
	rules := string(results[1])
	server.Rules = rules
	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("new rules: \n```markdown\n%s\n```", server.Rules))
	db.Save(&server)
}

func setRulesChannel(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	channel := fields[0]
	re := regexp.MustCompile("<#(.+)>")
	results := re.FindSubmatch([]byte(channel))
	if len(results) != 2 {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", channel))
	}
	channelID := string(results[1])
	server.RulesChannel = channelID
	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules in channel <#%s>", server.RulesChannel))
	db.Save(&server)
}

func setLogsChannel(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	channel := fields[0]
	re := regexp.MustCompile("<#(.*)>")
	results := re.FindSubmatch([]byte(channel))
	if len(results) != 2 {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", channel))
	}
	channelID := string(results[1])
	if channelID == "" {
		DiscordSession.ChannelMessageSend(m.ChannelID, "stopping logging user interractions")
	} else {
		DiscordSession.ChannelMessageSend(channelID, "initialized logs")
	}
	server.LogChannelID = channelID
	db.Save(&server)
}

func setReactions(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 2 {
		return
	}
	server.ReactionOk = fields[0]
	server.ReactionNo = fields[1]
	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("reactions set to %s / %s", server.ReactionOk, server.ReactionNo))
	db.Save(&server)
}

func setRole(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	role := fields[0]
	re := regexp.MustCompile("<@&(.*)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed role %s", role))
		return
	}
	roleID := string(results[1])
	server.Role = roleID
	if server.Role == "" {
		DiscordSession.ChannelMessageSend(m.ChannelID, "cleared allowed user role")
	} else {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules bot gives role %s", server.Role))
	}
	db.Save(&server)
}

func setAdminRole(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	role := fields[0]
	re := regexp.MustCompile("<@&(.*)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", role))
		return
	}
	roleID := string(results[1])
	server.AdminRole = roleID
	if server.AdminRole == "" {
		DiscordSession.ChannelMessageSend(m.ChannelID, "cleared admin user role")
	} else {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("new admin role: %s", server.AdminRole))
	}
	db.Save(&server)
}

func getRulesFromMessage(m *discordgo.Message) (string, error) {
	if m.Content != "" {
		return m.Content, nil
	}
	for _, embeded := range m.Embeds {
		if embeded.Description != "" {
			return embeded.Description, nil
		}
	}
	return "", errors.New("no appropriate content in the message")
}

func setRuleMessageID(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	messageID := fields[0]
	message, err := DiscordSession.ChannelMessage(server.RulesChannel, messageID)
	if err != nil {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("cant find message `%s` in channel <#%s>", messageID, server.RulesChannel))
		return
	}
	if rules, err := getRulesFromMessage(message); err == nil {
		server.Rules = rules
	}
	server.RulesMessageID = message.ID

	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("tracking rules on message: %s\nnew rules: \n```markdown\n%s\n```", server.RulesMessageID, server.Rules))
	db.Save(&server)
}

func setStrict(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	strictstring := fields[0]
	server.Strict = !(strictstring == "False" || strictstring == "false")
	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("strict mode: %t", server.Strict))
	db.Save(&server)
}

func setTest(server *Server, m *discordgo.Message, fields []string) {
	if len(fields) != 1 {
		return
	}
	teststring := fields[0]
	server.Test = !(teststring == "False" || teststring == "false")
	DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("test mode: %t", server.Test))
	db.Save(&server)
}

/*
* enable / disable the bot.
*
* adds the message as embeded, the emojis, and register the bot to add role for users
 */
func enableRules(server *Server, m *discordgo.Message, fields []string) {
	if server.Active {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules already active in <#%s>", server.RulesChannel))
		return
	}
	server.Active = true
	if server.RulesMessageID == "" {
		message, _ := DiscordSession.ChannelMessageSendEmbed(server.RulesChannel, &discordgo.MessageEmbed{Description: server.Rules})
		server.RulesMessageID = message.ID
	} else {
		DiscordSession.ChannelMessageSend(m.ChannelID, fmt.Sprintf("re-using message in <#%s>", server.RulesChannel))
	}

	err1 := DiscordSession.MessageReactionAdd(server.RulesChannel, server.RulesMessageID, server.ReactionOk)
	err2 := DiscordSession.MessageReactionAdd(server.RulesChannel, server.RulesMessageID, server.ReactionNo)
	if err1 != nil || err2 != nil {
		DiscordSession.ChannelMessageSend(m.ChannelID, "error adding reactions to rules message")
	}
	db.Save(&server)
}

func disableRules(server *Server, m *discordgo.Message, fields []string) {
	if !server.Active {
		DiscordSession.ChannelMessageSend(m.ChannelID, "Rules not in place")
		return
	}
	server.Active = false
	DiscordSession.ChannelMessageSend(m.ChannelID, "Rules disabled")
	db.Save(&server)
}

func updateRules(server *Server, m *discordgo.Message, fields []string) {
	if !server.Active {
		DiscordSession.ChannelMessageSend(m.ChannelID, "Rules not in place")
		return
	}
	_, err := DiscordSession.ChannelMessageEditEmbed(
		server.RulesChannel, server.RulesMessageID, &discordgo.MessageEmbed{Description: server.Rules},
	)
	if err != nil {
		DiscordSession.ChannelMessageSend(m.ChannelID, "an error occured, pls contact mod")
	}
}

func showStatus(server *Server, m *discordgo.Message, fields []string) {

	DiscordSession.ChannelMessageSend(
		m.ChannelID,
		fmt.Sprintf(
			"server ID: %s\nRules Channel: <#%s>\nAccepted Role: %s\nLog Channel: <#%s>\nEmotes: %s / %s\nActive: %t\nRules MessageID: %s\nRules:\n```markdown\n%s\n```\n",
			server.GuildID, server.RulesChannel, server.Role, server.LogChannelID, server.ReactionOk, server.ReactionNo, server.Active, server.RulesMessageID, server.Rules,
		),
	)
}
