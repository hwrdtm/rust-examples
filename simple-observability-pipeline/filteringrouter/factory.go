package filteringrouter

import (
	"context"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/connector"
	"go.opentelemetry.io/collector/consumer"
)

const (
	defaultVal = "request.n"
	// this is the name used to refer to the connector in the config.yaml
	typeStr = "example"
)

// NewFactory creates a factory for example connector.
func NewFactory() connector.Factory {
	// OpenTelemetry connector factory to make a factory for connectors
	return connector.NewFactory(
		component.MustNewType("filteringrouter"),
		createDefaultConfig,
		connector.WithLogsToLogs(createLogsToLogsConnector, component.StabilityLevelAlpha),
		connector.WithLogsToTraces(createLogsToTracesConnector, component.StabilityLevelAlpha),
		connector.WithLogsToMetrics(createLogsToMetricsConnector, component.StabilityLevelAlpha),
	)
}

func createDefaultConfig() component.Config {
	return &Config{
		AttributeName: defaultVal,
	}
}

func createLogsToLogsConnector(ctx context.Context, params connector.CreateSettings, cfg component.Config, nextConsumer consumer.Logs) (connector.Logs, error) {
	c, err := newLogsConnector(params.Logger, cfg)
	if err != nil {
		return nil, err
	}
	c.logsConsumer = nextConsumer
	return c, nil
}

func createLogsToTracesConnector(ctx context.Context, params connector.CreateSettings, cfg component.Config, nextConsumer consumer.Traces) (connector.Logs, error) {
	c, err := newTracesConnector(params.Logger, cfg)
	if err != nil {
		return nil, err
	}
	c.tracesConsumer = nextConsumer
	return c, nil
}

func createLogsToMetricsConnector(ctx context.Context, params connector.CreateSettings, cfg component.Config, nextConsumer consumer.Metrics) (connector.Logs, error) {
	c, err := newMetricsConnector(params.Logger, cfg)
	if err != nil {
		return nil, err
	}
	c.metricsConsumer = nextConsumer
	return c, nil
}
