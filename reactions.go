package main

import (
	"errors"
	"fmt"
	"log"

	"github.com/bwmarrin/discordgo"
)

func logInterraction(s *discordgo.Session, m *discordgo.MessageReaction, server *Server, action string, event string) {
	log.Printf("%s: %s", m.UserID, action)
	if server.LogChannelID == "" {
		return
	}
	s.ChannelMessageSendEmbed(
		server.LogChannelID,
		&discordgo.MessageEmbed{
			Title:       action,
			Description: fmt.Sprintf("<@%s>: %s", m.UserID, event),
		},
	)
}

func giveRole(
	s *discordgo.Session,
	guild *discordgo.Guild,
	user *discordgo.User,
	role *discordgo.Role,
) {
}
func takeRole(
	s *discordgo.Session,
	guild *discordgo.Guild,
	user *discordgo.User,
	role *discordgo.Role,
) {
}
func kick(
	s *discordgo.Session,
	guild *discordgo.Guild,
	user *discordgo.User,
) {
}

func handleReactionOk(s *discordgo.Session, m *discordgo.MessageReaction, server *Server) {
	if err := s.GuildMemberRoleAdd(m.GuildID, m.UserID, server.Role); err != nil {
		log.Println(err)
		logInterraction(s, m, server, "add-role", "error settig the role")
		return
	}
	logInterraction(s, m, server, "add-role", "accepted the rules")
}

func handleReactionKo(s *discordgo.Session, m *discordgo.MessageReaction, server *Server) {
	if err := s.GuildMemberDeleteWithReason(m.GuildID, m.UserID, "rejected the server rules"); err != nil {
		log.Println(err)
		logInterraction(s, m, server, "kick", "error kicking user")
		return
	}
	logInterraction(s, m, server, "kick", "rejected the rules")
}

func handleReactionRemoveOk(s *discordgo.Session, m *discordgo.MessageReaction, server *Server) {
	if err := s.GuildMemberRoleRemove(m.GuildID, m.UserID, server.Role); err != nil {
		log.Println(err)
		logInterraction(s, m, server, "remove-role", "error kicking user")
		return
	}
	logInterraction(s, m, server, "remove-role", "un-accepted the rules")
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

func ReactionAddHandler(s *discordgo.Session, m *discordgo.MessageReactionAdd) {
	server := Server{}
	if err := getServer(&server, m.MessageReaction); err != nil {
		log.Println(err)
		return
	}

	if m.Emoji.Name == server.ReactionOk {
		handleReactionOk(s, m.MessageReaction, &server)
	}
	if m.Emoji.Name == server.ReactionNo {
		handleReactionKo(s, m.MessageReaction, &server)
	}
}

func ReactionRemoveHandler(s *discordgo.Session, m *discordgo.MessageReactionRemove) {
	server := Server{}
	if err := getServer(&server, m.MessageReaction); err != nil {
		log.Println(err)
		return
	}

	if m.Emoji.Name == server.ReactionOk {
		handleReactionRemoveOk(s, m.MessageReaction, &server)
	}
}
