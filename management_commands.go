package main

import (
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
	re := regexp.MustCompile("<#(.+)>")
	results := re.FindSubmatch([]byte(channel))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", channel))
	}
	channelID := string(results[1])
	s.ChannelMessageSend(channelID, "initialized logs")
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
	re := regexp.MustCompile("<@&(.+)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed role %s", role))
	}
	roleID := string(results[1])
	server.Role = roleID
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules bot gives role %s", server.Role))
	db.Save(&server)
}

func setAdminRole(server *Server, s *discordgo.Session, m *discordgo.Message, role string) {
	re := regexp.MustCompile("<#(.+)>")
	results := re.FindSubmatch([]byte(role))
	if len(results) != 2 {
		s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("malformed channel %s", role))
	}
	roleID := string(results[1])
	server.Role = roleID
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("rules in channel <#%s>", server.RulesChannel))
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
			"server ID: %s\nRules Channel: <#%s>\nAccepted Role: %s\nLog Channel: <#%s>\nEmotes: %s / %s\nActive: %t\nRules:\n```markdown\n%s\n```\n",
			server.GuildID, server.RulesChannel, server.Role, server.LogChannelID, server.ReactionOk, server.ReactionNo, server.Active, server.Rules,
		),
	)
}
func showHelp(s *discordgo.Session, channelID string) {
	s.ChannelMessageSend(
		channelID,
		fmt.Sprintf("Help !"),
	)
}

func MessageHandler(s *discordgo.Session, m *discordgo.MessageCreate) {
	if m.Author.ID == s.State.User.ID || !botTriggered(s.State.User.ID, m.Message) {
		return
	}
	guild, err := s.Guild(m.GuildID)
	if err != nil {
		log.Printf("command on invalid server %s: %s", m.GuildID, m.Content)
		return
	}
	if m.Author.ID != guild.OwnerID {
		return
	}

	// let the menthion be the rirst thing in the comment
	reg := regexp.MustCompile(fmt.Sprintf("<@!?(%s)>", s.State.User.ID))
	if reg.FindStringIndex(m.Content)[0] != 0 {
		return
	}

	log.Printf("%s: %s\n", m.Author.ID, m.Content)

	server := Server{}
	result := db.Where(Server{GuildID: m.GuildID}).FirstOrInit(&server)
	// result.RecordNotFound() if going back to `First` only instead of FirstOrInit
	if result.RecordNotFound() || db.NewRecord(server) {
		log.Printf("command on unregistered server %s: %s", m.GuildID, m.Content)
	} else if result.Error != nil {
		log.Println(result.Error)
		return
	}
	fields := strings.Fields(m.Content)

	// no point to continue if there's no fields
	if len(fields) <= 1 {
		return
	}

	switch fields[1] {

	case "set-rules":
		setRulesContent(&server, s, m.Message)
	case "set-rules-channel":
		setRulesChannel(&server, s, m.Message, fields[2])
	case "set-logs-channel":
		setLogsChannel(&server, s, m.Message, fields[2])
	case "set-reactions":
		if len(fields) != 4 {
			log.Println("bite", fields)
			return
		}
		setReactions(&server, s, m.Message, fields[2], fields[3])
	case "set-role":
		setRole(&server, s, m.Message, fields[2])
	case "set-admin-role":
		setRole(&server, s, m.Message, fields[2])

	case "disable":
		disableRules(&server, s, m.Message)
	case "update":
		updateRules(&server, s, m.Message)
	case "enable":
		enableRules(&server, s, m.Message)

	case "status":
		showStatus(server, s, m.Message)
		return
	default:
		showHelp(s, m.ChannelID)
		return
	}
}
