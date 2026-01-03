package api

import (
	"context"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"go.uber.org/zap"

	"srv-yoda/internal/graph"
	"srv-yoda/internal/kafka"
	"srv-yoda/internal/llm"
	"srv-yoda/internal/models"
)

type Handler struct {
	graph  *graph.GraphRetriever
	llm    *llm.GeminiClient
	kafka  *kafka.Producer
	logger *zap.Logger
}

func NewHandler(g *graph.GraphRetriever, l *llm.GeminiClient, k *kafka.Producer, logger *zap.Logger) *Handler {
	return &Handler{
		graph:  g,
		llm:    l,
		kafka:   k,
		logger: logger,
	}
}

func (h *Handler) HealthCheck(c *gin.Context) {
	c.JSON(http.StatusOK, gin.H{
		"status":   "ok",
		"service": "srv-yoda",
		"time":    time.Now(),
	})
}

func (h *Handler) QueryGraphRAG(c *gin.Context) {
	start := time.Now()
	ctx := context.Background()

	var req models.QueryRequest
	if err := c. ShouldBindJSON(&req); err != nil {
		c.JSON(http.StatusBadRequest, models.ErrorResponse{
			Error:   "invalid_request",
			Message: err.Error(),
		})
		return
	}

	queryID := uuid.New().String()
	h.logger.Info("New query received",
		zap.String("query_id", queryID),
		zap.String("question", req.Question),
	)

	if h.kafka != nil {
		_ = h.kafka.PublishEvent(ctx, "starwars. queries.incoming", queryID, map[string]interface{}{
			"query_id":   queryID,
			"question":   req.Question,
			"timestamp": time.Now().Unix(),
		})
	}

	graphCtx, err := h.graph.SearchContext(ctx, req.Question)
	if err != nil {
		h.logger.Error("Graph search failed", zap.Error(err))
		c.JSON(http.StatusInternalServerError, models.ErrorResponse{
			Error:    "graph_search_failed",
			Message: err.Error(),
		})
		return
	}

	answer, err := h.llm. GenerateAnswer(ctx, req. Question, graphCtx)
	if err != nil {
		h.logger.Error("LLM generation failed", zap.Error(err))
		c.JSON(http.StatusInternalServerError, models.ErrorResponse{
			Error:   "llm_generation_failed",
			Message: err. Error(),
		})
		return
	}

	sources := extractSources(graphCtx)
	response := models.RAGResponse{
		QueryID:       queryID,
		Answer:        answer,
		Context:       *graphCtx,
		Sources:        sources,
		ProcessTimeMs: time.Since(start).Milliseconds(),
	}

	if h.kafka != nil {
		_ = h.kafka.PublishEvent(ctx, "starwars. analytics", queryID, map[string]interface{}{
			"query_id":        queryID,
			"process_time_ms": response.ProcessTimeMs,
			"nodes_found":     len(graphCtx. Nodes),
			"sources_count":   len(sources),
		})
	}

	h.logger.Info("Query processed",
		zap.String("query_id", queryID),
		zap.Int64("time_ms", response.ProcessTimeMs),
	)

	c.JSON(http.StatusOK, response)
}

func extractSources(ctx *models.GraphContext) []string {
	sources := make([]string, 0, len(ctx.Nodes))
	seen := make(map[string]bool)

	for _, node := range ctx.Nodes {
		if !seen[node.Name] {
			sources = append(sources, node.Name)
			seen[node.Name] = true
		}
	}

	return sources
}
