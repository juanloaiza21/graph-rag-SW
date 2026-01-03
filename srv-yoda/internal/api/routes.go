package api

import (
	"github.com/gin-gonic/gin"
	"go.uber.org/zap"

	"srv-yoda/internal/graph"
	"srv-yoda/internal/kafka"
	"srv-yoda/internal/llm"
)

func SetupRouter(
	graphRetriever *graph.GraphRetriever,
	llmClient *llm.GeminiClient,
	kafkaProducer *kafka.Producer,
	logger *zap.Logger,
) *gin.Engine {
	gin.SetMode(gin.ReleaseMode)
	router := gin.New()

	router.Use(gin.Recovery())
	router.Use(ginLogger(logger))
	router.Use(corsMiddleware())

	handler := NewHandler(graphRetriever, llmClient, kafkaProducer, logger)

	router.GET("/health", handler.HealthCheck)
	router.POST("/query", handler.QueryGraphRAG)

	v1 := router.Group("/api/v1")
	{
		v1.GET("/health", handler.HealthCheck)
		v1.POST("/query", handler.QueryGraphRAG)
	}

	return router
}

func ginLogger(logger *zap.Logger) gin.HandlerFunc {
	return func(c *gin.Context) {
		logger.Info("Incoming request",
			zap. String("method", c.Request. Method),
			zap.String("path", c.Request.URL.Path),
			zap.String("ip", c.ClientIP()),
		)
		c.Next()
	}
}

func corsMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}

		c.Next()
	}
}
