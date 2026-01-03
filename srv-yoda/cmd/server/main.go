package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"go.uber.org/zap"

	"srv-yoda/internal/api"
	"srv-yoda/internal/config"
	"srv-yoda/internal/graph"
	"srv-yoda/internal/kafka"
	"srv-yoda/internal/llm"
	"srv-yoda/pkg/logger"
)

func main() {
	cfg := config.Load()

	log, err := logger.NewLogger(cfg.LogLevel)
	if err != nil {
		fmt.Printf("Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer log.Sync()

	log.Info("🧘 Starting srv-yoda",
		zap.String("port", cfg.Port),
		zap.String("neo4j", cfg.Neo4jURI),
	)

	graphRetriever, err := graph.NewGraphRetriever(cfg.Neo4jURI, cfg.Neo4jUser, cfg.Neo4jPassword)
	if err != nil {
		log.Fatal("Failed to connect to Neo4j", zap. Error(err))
	}
	defer graphRetriever.Close(context.Background())
	log.Info("✅ Connected to Neo4j")

	if cfg.GeminiAPIKey == "" {
		log. Warn("⚠️  GEMINI_API_KEY not set, LLM features will be disabled")
	}
	llmClient := llm.NewGeminiClient(cfg.GeminiAPIKey)
	log.Info("✅ Gemini client initialized")

	var kafkaProducer *kafka. Producer
	if len(cfg.KafkaBrokers) > 0 && cfg.KafkaBrokers[0] != "" {
		kafkaProducer = kafka.NewProducer(cfg.KafkaBrokers)
		defer kafkaProducer.Close()
		log.Info("✅ Kafka producer initialized", zap.Strings("brokers", cfg.KafkaBrokers))
	} else {
		log.Warn("⚠️  Kafka not configured, events will not be published")
	}

	router := api.SetupRouter(graphRetriever, llmClient, kafkaProducer, log)

	srv := &http.Server{
		Addr:         ":" + cfg.Port,
		Handler:      router,
		ReadTimeout:  30 * time.Second,
		WriteTimeout: 30 * time.Second,
		IdleTimeout:  60 * time.Second,
	}

	go func() {
		log.Info("🚀 Server listening", zap.String("address", srv.Addr))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatal("Server failed to start", zap.Error(err))
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall. SIGINT, syscall.SIGTERM)
	<-quit

	log.Info("🛑 Shutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		log.Fatal("Server forced to shutdown", zap.Error(err))
	}

	log.Info("✅ Server stopped gracefully")
}
