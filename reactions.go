package main

import (
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
	}
	logInterraction(s, m, server, "add-role", "accepted the rules")
}

func handleReactionKo(s *discordgo.Session, m *discordgo.MessageReaction, server *Server) {
	s.GuildMemberDeleteWithReason(m.GuildID, m.UserID, "rejected the server rules")
	logInterraction(s, m, server, "kick", "rejected the rules")
}

func handleRemoveOk(s *discordgo.Session, m *discordgo.MessageReaction, server *Server) {
	s.GuildMemberRoleRemove(m.GuildID, m.UserID, server.Role)
	logInterraction(s, m, server, "remove-role", "un-accepted the rules")
}

func ReactionHandler(s *discordgo.Session, m *discordgo.MessageReactionAdd) {
	server := Server{}
	result := db.Where(Server{GuildID: m.GuildID}).FirstOrInit(&server)

	if result.RecordNotFound() || db.NewRecord(server) || !server.Active || m.MessageID != server.RulesMessageID {
		return
	} else if result.Error != nil {
		log.Println(result.Error)
		return
	}

	if m.Emoji.Name == server.ReactionOk {
		handleReactionOk(s, m.MessageReaction, &server)
	}
	if m.Emoji.Name == server.ReactionNo {
		handleReactionKo(s, m.MessageReaction, &server)
	}
}
