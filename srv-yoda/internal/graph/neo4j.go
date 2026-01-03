package graph

import (
	"context"
	"fmt"
	"strings"

	"github.com/neo4j/neo4j-go-driver/v5/neo4j"
	"srv-yoda/internal/models"
)

type GraphRetriever struct {
	driver neo4j. DriverWithContext
}

func NewGraphRetriever(uri, user, password string) (*GraphRetriever, error) {
	driver, err := neo4j.NewDriverWithContext(uri, neo4j.BasicAuth(user, password, ""))
	if err != nil {
		return nil, fmt. Errorf("error creating Neo4j driver: %w", err)
	}

	ctx := context.Background()
	if err := driver.VerifyConnectivity(ctx); err != nil {
		return nil, fmt. Errorf("failed to verify Neo4j connectivity: %w", err)
	}

	return &GraphRetriever{driver:  driver}, nil
}

func (g *GraphRetriever) SearchContext(ctx context.Context, question string) (*models.GraphContext, error) {
	session := g.driver.NewSession(ctx, neo4j.SessionConfig{DatabaseName: "neo4j"})
	defer session.Close(ctx)

	entities := extractEntities(question)

	if len(entities) == 0 {
		return &models.GraphContext{
			Nodes:         []models.Node{},
			Relationships: []models.Relationship{},
		}, nil
	}

	query := `
		MATCH (n)
		WHERE n.name IN $entities
		OPTIONAL MATCH path = (n)-[r*1..2]-(related)
		WITH n, nodes(path) as pathNodes
		UNWIND pathNodes as node
		RETURN DISTINCT 
			node. id as id,
			labels(node)[0] as label,
			node.name as name,
			properties(node) as props
		LIMIT 20
	`

	result, err := session.Run(ctx, query, map[string]interface{}{
		"entities": entities,
	})
	if err != nil {
		return nil, fmt.Errorf("graph query failed: %w", err)
	}

	graphCtx := &models.GraphContext{
		Nodes:         []models.Node{},
		Relationships: []models.Relationship{},
	}

	for result.Next(ctx) {
		record := result.Record()

		id, _ := record.Get("id")
		label, _ := record.Get("label")
		name, _ := record.Get("name")
		props, _ := record.Get("props")

		node := models.Node{
			ID:         fmt.Sprintf("%v", id),
			Label:      fmt.Sprintf("%v", label),
			Name:       fmt. Sprintf("%v", name),
			Properties: props.(map[string]interface{}),
		}

		graphCtx. Nodes = append(graphCtx.Nodes, node)
	}

	if err := result.Err(); err != nil {
		return nil, fmt.Errorf("error iterating results: %w", err)
	}

	return graphCtx, nil
}

func (g *GraphRetriever) Close(ctx context.Context) error {
	return g.driver.Close(ctx)
}

func extractEntities(text string) []string {
	knownEntities := []string{
		"Luke Skywalker", "Darth Vader", "Leia Organa", "Han Solo",
		"Yoda", "Obi-Wan Kenobi", "Anakin Skywalker", "Palpatine",
		"Chewbacca", "R2-D2", "C-3PO", "Padmé Amidala",
		"Tatooine", "Alderaan", "Hoth", "Dagobah", "Endor",
		"Death Star", "Millennium Falcon", "X-wing",
	}

	found := []string{}
	textLower := strings.ToLower(text)

	for _, entity := range knownEntities {
		if strings.Contains(textLower, strings.ToLower(entity)) {
			found = append(found, entity)
		}
	}

	return found
}
