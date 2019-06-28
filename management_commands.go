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

func setRuleMessageID(server *Server, s *discordgo.Session, m *discordgo.Message, messageID string) {
	server.RulesMessageID = messageID
	s.ChannelMessageSend(m.ChannelID, fmt.Sprintf("tracking rules on message: %s", messageID))
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

func MessageHandler(s *discordgo.Session, m *discordgo.MessageCreate) {
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

	switch fields[1] {

	case "set-rules":
		setRulesContent(&server, s, m.Message)
	case "set-rules-channel":
		if len(fields) != 3 {
			return
		}
		setRulesChannel(&server, s, m.Message, fields[2])
	case "set-logs-channel":
		if len(fields) != 3 {
			return
		}
		setLogsChannel(&server, s, m.Message, fields[2])
	case "set-reactions":
		if len(fields) != 4 {
			log.Println("missing set-reaction-args", fields)
			return
		}
		setReactions(&server, s, m.Message, fields[2], fields[3])
	case "set-role":
		if len(fields) != 3 {
			return
		}
		setRole(&server, s, m.Message, fields[2])
	case "set-admin-role":
		if len(fields) != 3 {
			return
		}
		setAdminRole(&server, s, m.Message, fields[2])
	case "set-message-id":
		if len(fields) != 3 {
			return
		}
		setRuleMessageID(&server, s, m.Message, fields[2])

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
