package main

import (
	"errors"
	"fmt"
	"log"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

func botTriggered(botID string, m *discordgo.Message) bool {
	for _, user := range m.Mentions {
		if user.ID == botID {
			return true
		}
	}
	return false
}

/*
 * configure the bot (rules, channels for rules and logs, change reactions, etc...
 */
func setRulesContent(server *Server, s *discordgo.Session, m *discordgo.Message) {
	re := regexp.MustCompile("(?s)```\n?(.+)\n?```")
	results := re.FindSubmatch([]byte(m.Content))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, "malformed rules")
		return
	}
	rules := string(results[1])
	server.Rules = rules
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("new rules: \n```markdown\n%s\n```", server.Rules))
	db.Save(&server)
}

func setRulesChannel(server *Server, s *discordgo.Session, m *discordgo.Message, channel string) {
	re := regexp.MustCompile("<#(.+)>")
	results := re.FindSubmatch([]byte(channel))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", channel))
	}
	channelID := string(results[1])
	server.RulesChannel = channelID
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules in channel <#%s>", server.RulesChannel))
	db.Save(&server)
}

func setLogsChannel(server *Server, s *discordgo.Session, m *discordgo.Message, channel string) {
	re := regexp.MustCompile("<#(.*)>")
	results := re.FindSubmatch([]byte(channel))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", channel))
	}
	channelID := string(results[1])
	if channelID == "" {
		s.ChannelMessageSend(m.ChannelID, "stopping logging user interractions")
	} else {
		s.ChannelMessageSend(channelID, "initialized logs")
	}
	server.LogChannelID = channelID
	db.Save(&server)
}

func setReactions(server *Server, s *discordgo.Session, m *discordgo.Message, reac1 string, reac2 string) {
	server.ReactionOk = reac1
	server.ReactionNo = reac2
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("reactions set to %s / %s", server.ReactionOk, server.ReactionNo))
	db.Save(&server)
}

func setRole(server *Server, s *discordgo.Session, m *discordgo.Message, role string) {
	re := regexp.MustCompile("<@&(.*)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed role %s", role))
		return
	}
	roleID := string(results[1])
	server.Role = roleID
	if server.Role == "" {
		s.ChannelMessageSend(m.ChannelID, "cleared allowed user role")
	} else {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules bot gives role %s", server.Role))
	}
	db.Save(&server)
}

func setAdminRole(server *Server, s *discordgo.Session, m *discordgo.Message, role string) {
	re := regexp.MustCompile("<@&(.*)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", role))
		return
	}
	roleID := string(results[1])
	server.AdminRole = roleID
	if server.Role == "" {
		s.ChannelMessageSend(m.ChannelID, "cleared admin user role")
	} else {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("new admin role: %s", server.AdminRole))
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

func setRuleMessageID(server *Server, s *discordgo.Session, m *discordgo.Message, messageID string) {
	message, err := s.ChannelMessage(server.RulesChannel, messageID)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("cant find message `%s` in channel <#%s>", messageID, server.RulesChannel))
		return
	}
	if rules, err := getRulesFromMessage(message); err == nil {
		server.Rules = rules
	}
	server.RulesMessageID = message.ID

	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("tracking rules on message: %s\nnew rules: \n```markdown\n%s\n```", server.RulesMessageID, server.Rules))
	db.Save(&server)
}

func setStrict(server *Server, s *discordgo.Session, m *discordgo.Message, strictstring string) {
	server.Strict = !(strictstring == "False" || strictstring == "false")
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("strict mode: %t", server.Strict))
	db.Save(&server)
}

func setTest(server *Server, s *discordgo.Session, m *discordgo.Message, teststring string) {
	server.Test = !(teststring == "False" || teststring == "false")
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("test mode: %t", server.Test))
	db.Save(&server)
}

/*
* enable / disable the bot.
*
* adds the message as embeded, the emojis, and register the bot to add role for users
 */
func enableRules(server *Server, s *discordgo.Session, m *discordgo.Message) {
	if server.Active {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules already active in <#%s>", server.RulesChannel))
		return
	}
	server.Active = true
	message, _ := s.ChannelMessageSendEmbed(server.RulesChannel, &discordgo.MessageEmbed{Description: server.Rules})
	server.RulesMessageID = message.ID

	err1 := s.MessageReactionAdd(server.RulesChannel, server.RulesMessageID, server.ReactionOk)
	err2 := s.MessageReactionAdd(server.RulesChannel, server.RulesMessageID, server.ReactionNo)
	if err1 != nil || err2 != nil {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules already active in <#%s>", server.RulesChannel))
	}
	db.Save(&server)
}

func disableRules(server *Server, s *discordgo.Session, m *discordgo.Message) {
	if !server.Active {
		s.ChannelMessageSend(m.ChannelID, "Rules not in place")
		return
	}
	server.Active = false
	s.ChannelMessageSend(m.ChannelID, "Rules disabled")
	db.Save(&server)
}

func updateRules(server *Server, s *discordgo.Session, m *discordgo.Message) {
	if !server.Active {
		s.ChannelMessageSend(m.ChannelID, "Rules not in place")
		return
	}
	_, err := s.ChannelMessageEditEmbed(
		server.RulesChannel, server.RulesMessageID, &discordgo.MessageEmbed{Description: server.Rules},
	)
	if err != nil {
		s.ChannelMessageSend(m.ChannelID, "an error occured, pls contact mod")
	}
}

func showStatus(server Server, s *discordgo.Session, m *discordgo.Message) {

	s.ChannelMessageSend(
		m.ChannelID,
		fmt.Sprintf(
			"server ID: %s\nRules Channel: <#%s>\nAccepted Role: %s\nLog Channel: <#%s>\nEmotes: %s / %s\nActive: %t\nRules MessageID: %s\nRules:\n```markdown\n%s\n```\n",
			server.GuildID, server.RulesChannel, server.Role, server.LogChannelID, server.ReactionOk, server.ReactionNo, server.Active, server.RulesMessageID, server.Rules,
		),
	)
}
func showHelp(s *discordgo.Session, channelID string) {
	s.ChannelMessageSend(
		channelID,
		fmt.Sprintf(
			"```\n"+
				"# Commands:\n"+
				"- `set-rules `\u200B`\u200B`<RULES>`\u200B`\u200B` `: set the rules, put them inside a block code (markdown allowed)\n"+
				"- `set-rules-channel #<CHANNEL>`: set the channel the bot will post its message in\n"+
				"- `set-logs-channel #<CHANNEL>`: set the channel the bot will log its interractions with users in\n"+
				"- `set-reactions <OK> <KO>`: set the emojis used for yes/no answers if you want to change them\n"+
				"- `set-role @<ROLE>`: set the role the bot will give to your users after they accept the rules\n"+
				"- `set-admin-role @<ROLE>`: set the role that will be allowed to interract with the bot (admins/moderators)\n"+
				"- `set-message-id <MESSAGE_ID>`: set the ID of the message the bot will track to give / take permission\n"+
				"\n"+
				"- `enable`: put the rules message in the rules channel, start tracking reactions and assigning role\n"+
				"- `disable`: stop tracking the reactions to the rules message\n"+
				"- `update`: apply recent rules change to the \n"+
				"- `status`: show configuration\n"+
				"```",
		),
	)
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
