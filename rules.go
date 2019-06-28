package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/bwmarrin/discordgo"
	"github.com/jinzhu/gorm"
	_ "github.com/jinzhu/gorm/dialects/sqlite"
)

type Server struct {
	gorm.Model
	GuildID        string
	Role           string
	AdminRole      string
	Rules          string
	RulesChannel   string
	LogChannelID   string
	RulesMessageID string
	ReactionOk     string `gorm:"default:'\u2705'"`
	ReactionNo     string `gorm:"default:'\u274C'"`
	Active         bool
	Strict         bool
}

var (
	db_dialect        string
	db_connection     string
	db                *gorm.DB
	DiscordToken      string
	DiscordSession, _ = discordgo.New()
)

func init() {
	flag.StringVar(&DiscordToken, "t", "", "Discord Authentication Token")
	flag.StringVar(&db_dialect, "db-dialect", "sqlite3", "db dialect (sqlite3 / postgres)")
	flag.StringVar(&db_connection, "db", ":memory:", "db connection info")
}

func main() {
	var err error
	flag.Parse()
	db, err = gorm.Open(db_dialect, db_connection)
	if err != nil {
		log.Fatal(err)
	}
	defer db.Close()

	db.AutoMigrate(&Server{})

	DiscordSession.Token = "Bot " + DiscordToken
	DiscordSession.State.User, err = DiscordSession.User("@me")
	if err != nil {
		log.Fatalf("error fetching user information, %s\n", err)
	}
	DiscordSession.AddHandler(MessageHandler)
	DiscordSession.AddHandler(ReactionHandler)
	if err := DiscordSession.Open(); err != nil {
		log.Fatalf("error opening connection to Discord, %s\n", err)
	}
	defer DiscordSession.Close()
	log.Println(`Now running. Press CTRL-C to exit.`)
	sc := make(chan os.Signal, 1)
	signal.Notify(sc, syscall.SIGINT, syscall.SIGTERM, os.Interrupt, os.Kill)
	<-sc
}
