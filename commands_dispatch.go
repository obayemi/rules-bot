package main

import (
	"fmt"
	"log"
	"regexp"
	"strings"

	"github.com/bwmarrin/discordgo"
)

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
	//case "set-strict":
	//if len(fields) != 3 {
	//return
	//}
	//setStrict(&server, s, m.Message, fields[2])
	case "set-test":
		if len(fields) != 3 {
			return
		}
		setTest(&server, s, m.Message, fields[2])

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
