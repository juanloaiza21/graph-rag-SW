package config

import (
	"log"
	"os"
	"path/filepath"
	"strconv"

	"github.com/joho/godotenv"
)

type Config struct {
	Neo4jURI      string
	Neo4jUser     string
	Neo4jPassword string
	GeminiAPIKey  string
	KafkaBrokers  []string
	KafkaTopics   KafkaTopics
	RedisURI      string
	RedisCacheTTL int
	Port          string
	LogLevel      string
}

type KafkaTopics struct {
	QueriesIncoming  string
	QueriesResponses string
	Analytics        string
	GraphUpdates     string
}

func Load() *Config {
	loadEnvFromRoot()

	cacheTTL, _ := strconv.Atoi(getEnv("REDIS_CACHE_TTL", "600"))

	return &Config{
		Neo4jURI:      getEnv("NEO4J_URI", "bolt://localhost:7687"),
		Neo4jUser:     getEnv("NEO4J_USER", "neo4j"),
		Neo4jPassword: getEnv("NEO4J_PASSWORD", "password"),
		GeminiAPIKey:  getEnv("GEMINI_API_KEY", getEnv("GOOGLE_API_KEY", "")),
		KafkaBrokers:  []string{getEnv("KAFKA_BROKERS", "localhost: 9092")},
		KafkaTopics: KafkaTopics{
			QueriesIncoming:  getEnv("KAFKA_TOPIC_QUERIES_INCOMING", "starwars. queries.incoming"),
			QueriesResponses: getEnv("KAFKA_TOPIC_QUERIES_RESPONSES", "starwars.queries.responses"),
			Analytics:        getEnv("KAFKA_TOPIC_ANALYTICS", "starwars.analytics"),
			GraphUpdates:     getEnv("KAFKA_TOPIC_GRAPH_UPDATES", "starwars.graph.updates"),
		},
		RedisURI:       getEnv("REDIS_URI", "redis://:redis_pass_123@localhost:6379/0"),
		RedisCacheTTL: cacheTTL,
		Port:          getEnv("YODA_PORT", "8080"),
		LogLevel:      getEnv("LOG_LEVEL", "info"),
	}
}

func loadEnvFromRoot() {
	if err := godotenv.Load(); err == nil {
		log. Println("✅ .env loaded from current directory")
		return
	}

	rootEnv := filepath.Join(". .", ". env")
	if err := godotenv.Load(rootEnv); err == nil {
		log.Println("✅ .env loaded from parent directory")
		return
	}

	rootEnv2 := filepath.Join(". .", ". .", ".env")
	if err := godotenv.Load(rootEnv2); err == nil {
		log.Println("✅ .env loaded from root directory")
		return
	}

	log.Println("⚠️  No . env file found, using system environment variables")
}

func getEnv(key, defaultValue string) string {
	value := os.Getenv(key)
	if value == "" {
		return defaultValue
	}
	return value
}
