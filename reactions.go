package main

import (
	"errors"
	"fmt"
	"log"

	"github.com/bwmarrin/discordgo"
)

func logInterraction(s *discordgo.Session, userID string, server *Server, action string, event string) {
	log.Printf("%s: %s", userID, action)
	if server.LogChannelID == "" {
		return
	}
	if server.Test {
		action += "(DEBUG)"
	}
	s.ChannelMessageSendEmbed(
		server.LogChannelID,
		&discordgo.MessageEmbed{
			Title:       action,
			Description: fmt.Sprintf("<@%s>: %s", userID, event),
		},
	)
}

func handleReactionOk(s *discordgo.Session, userID string, server *Server) {
	if !server.Test {
		if err := s.GuildMemberRoleAdd(server.GuildID, userID, server.Role); err != nil {
			log.Println(err)
			logInterraction(s, userID, server, "add-role", "error settig the role")
			return
		}
	}
	logInterraction(s, userID, server, "add-role", "accepted the rules")
}

func handleReactionKo(s *discordgo.Session, userID string, server *Server) {
	if !server.Test {
		if err := s.GuildMemberDeleteWithReason(server.GuildID, userID, "rejected the server rules"); err != nil {
			log.Println(err)
			logInterraction(s, userID, server, "kick", "error kicking user")
			return
		}
	}
	logInterraction(s, userID, server, "kick", "rejected the rules")
}

func handleReactionRemoveOk(s *discordgo.Session, userID string, server *Server) {
	if !server.Test {
		if err := s.GuildMemberRoleRemove(server.GuildID, userID, server.Role); err != nil {
			log.Println(err)
			logInterraction(s, userID, server, "remove-role", "error removing user permission")
			return
		}
	}
	logInterraction(s, userID, server, "remove-role", "un-accepted the rules")
}

func getServer(server *Server, m *discordgo.MessageReaction) error {
	result := db.Where(Server{GuildID: m.GuildID}).FirstOrInit(&server)

	if result.RecordNotFound() || db.NewRecord(server) || !server.Active || m.MessageID != server.RulesMessageID {
		return errors.New("server not found")
	} else if result.Error != nil {
		return result.Error
	}
	return nil
}

func reactionAddHandler(s *discordgo.Session, m *discordgo.MessageReactionAdd) {
	server := Server{}
	if err := getServer(&server, m.MessageReaction); err != nil {
		log.Println(err)
		return
	}

	if m.Emoji.Name == server.ReactionOk {
		handleReactionOk(s, m.UserID, &server)
	}
	if m.Emoji.Name == server.ReactionNo {
		handleReactionKo(s, m.UserID, &server)
	}
}

func reactionRemoveHandler(s *discordgo.Session, m *discordgo.MessageReactionRemove) {
	server := Server{}
	if err := getServer(&server, m.MessageReaction); err != nil {
		log.Println(err)
		return
	}

	if m.Emoji.Name == server.ReactionOk {
		handleReactionRemoveOk(s, m.UserID, &server)
	}
}
