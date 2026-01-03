package llm

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"

	"srv-yoda/internal/models"
)

type GeminiClient struct {
	apiKey     string
	httpClient *http.Client
}

func NewGeminiClient(apiKey string) *GeminiClient {
	return &GeminiClient{
		apiKey:     apiKey,
		httpClient: &http.Client{},
	}
}

type geminiRequest struct {
	Contents []geminiContent `json:"contents"`
}

type geminiContent struct {
	Parts []geminiPart `json:"parts"`
}

type geminiPart struct {
	Text string `json:"text"`
}

type geminiResponse struct {
	Candidates []struct {
		Content struct {
			Parts []geminiPart `json:"parts"`
		} `json:"content"`
	} `json:"candidates"`
}

func (g *GeminiClient) GenerateAnswer(ctx context.Context, question string, graphCtx *models.GraphContext) (string, error) {
	prompt := buildPrompt(question, graphCtx)

	reqBody := geminiRequest{
		Contents: []geminiContent{{
			Parts: []geminiPart{{Text: prompt}},
		}},
	}

	jsonData, err := json.Marshal(reqBody)
	if err != nil {
		return "", fmt.Errorf("error marshaling request: %w", err)
	}

	url := fmt.Sprintf(
		"https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash: generateContent?key=%s",
		g.apiKey,
	)

	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewBuffer(jsonData))
	if err != nil {
		return "", fmt. Errorf("error creating request:  %w", err)
	}
	req.Header.Set("Content-Type", "application/json")

	resp, err := g.httpClient.Do(req)
	if err != nil {
		return "", fmt.Errorf("error calling Gemini API: %w", err)
	}
	defer resp.Body.Close()

	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("error reading response: %w", err)
	}

	if resp.StatusCode != 200 {
		return "", fmt. Errorf("Gemini API error %d: %s", resp.StatusCode, string(body))
	}

	var geminiResp geminiResponse
	if err := json.Unmarshal(body, &geminiResp); err != nil {
		return "", fmt.Errorf("error parsing response: %w", err)
	}

	if len(geminiResp.Candidates) == 0 || len(geminiResp.Candidates[0].Content.Parts) == 0 {
		return "", fmt.Errorf("empty response from Gemini")
	}

	return geminiResp.Candidates[0].Content.Parts[0]. Text, nil
}

func buildPrompt(question string, ctx *models.GraphContext) string {
	var sb strings.Builder

	sb.WriteString("You are a Star Wars expert assistant. Answer the question using ONLY the provided knowledge graph.\n\n")

	if len(ctx.Nodes) > 0 {
		sb. WriteString("## Available Entities:\n")
		for _, node := range ctx.Nodes {
			sb. WriteString(fmt.Sprintf("- **%s** (%s)\n", node.Name, node.Label))

			if desc, ok := node.Properties["wiki_description"].(string); ok && desc != "" {
				if len(desc) > 200 {
					desc = desc[: 200] + "..."
				}
				sb.WriteString(fmt.Sprintf("  Description: %s\n", desc))
			}
		}
		sb.WriteString("\n")
	}

	sb.WriteString(fmt.Sprintf("## Question:\n%s\n\n", question))

	sb.WriteString("## Instructions:\n")
	sb.WriteString("- Answer based ONLY on the entities listed above\n")
	sb.WriteString("- Cite specific entities in your answer\n")
	sb.WriteString("- If information is insufficient, state that clearly\n")
	sb.WriteString("- Be concise but informative\n\n")
	sb.WriteString("Answer:")

	return sb.String()
}
