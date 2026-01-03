package models

// QueryRequest es la petición del usuario
type QueryRequest struct {
	Question string `json:"question" binding:"required"`
}

// Node representa un nodo del grafo
type Node struct {
	ID         string                 `json:"id"`
	Label      string                 `json:"label"`
	Name       string                 `json:"name"`
	Properties map[string]interface{} `json:"properties,omitempty"`
}

// Relationship representa una relación entre nodos
type Relationship struct {
	Type   string `json:"type"`
	Source string `json:"source"`
	Target string `json:"target"`
}

// GraphContext es el contexto extraído del grafo
type GraphContext struct {
	Nodes         []Node         `json:"nodes"`
	Relationships []Relationship `json:"relationships"`
}

// RAGResponse es la respuesta final al usuario
type RAGResponse struct {
	QueryID       string       `json:"query_id"`
	Answer        string       `json:"answer"`
	Context       GraphContext `json:"context"`
	Sources       []string     `json:"sources"`
	ProcessTimeMs int64        `json:"process_time_ms"`
}

// ErrorResponse es la respuesta en caso de error
type ErrorResponse struct {
	Error   string `json:"error"`
	Message string `json:"message,omitempty"`
}
