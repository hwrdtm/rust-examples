package filteringrouter

import (
	"context"
	"errors"
	"fmt"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.opentelemetry.io/collector/pdata/pmetric"
	"go.uber.org/zap"
)

// schema for connector
type metricsConnectorImp struct {
	config          Config
	metricsConsumer consumer.Metrics
	logger          *zap.Logger
	// Include these parameters if a specific implementation for the Start and Shutdown function are not needed
	component.StartFunc
	component.ShutdownFunc
}

func newMetricsConnector(logger *zap.Logger, config component.Config) (*metricsConnectorImp, error) {
	logger.Info("Building filteringrouter metrics connector")
	cfg := config.(*Config)

	return &metricsConnectorImp{
		config: *cfg,
		logger: logger,
	}, nil
}

// Capabilities implements the consumer interface.
func (c *metricsConnectorImp) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

func (c *metricsConnectorImp) ConsumeLogs(ctx context.Context, ld plog.Logs) error {
	// When reading resourceMetrics from a log line, we assume that the resourceMetrics
	// string is in the body of each logRecord body.
	for i := 0; i < ld.ResourceLogs().Len(); i++ {
		resourceLog := ld.ResourceLogs().At(i)
		scopeLog := resourceLog.ScopeLogs().At(0)

		numLogRecords := scopeLog.LogRecords().Len()
		allErrs := make([]error, numLogRecords)

		// We continue to process the next log record even if there is an error in the current one
		// because errors in processing one log record should not prevent the processing of other log records.
		for j := 0; j < numLogRecords; j++ {
			logRecord := scopeLog.LogRecords().At(j)
			logBodyStr := logRecord.Body().AsString()

			// Do nothing if the log body string does not start with {"resourceMetrics"
			if err := validateStringBeginsWith(logBodyStr, "{\"resourceMetrics\":"); err != nil {
				c.logger.Debug("Log body string validation failed", zap.Error(err))
				allErrs[j] = nil
				continue
			}

			// Try to parse the log body as a pmetric.Metrics
			unmarshaler := pmetric.JSONUnmarshaler{}
			logBodyBytes := []byte(logBodyStr)
			parsedMetric, unmarshalErr := unmarshaler.UnmarshalMetrics(logBodyBytes)
			if unmarshalErr != nil {
				c.logger.Error("Error unmarshalling metrics", zap.Error(unmarshalErr))
				allErrs[j] = unmarshalErr
				continue
			}

			// Pass the parsed metric to the next consumer
			consumeErr := c.metricsConsumer.ConsumeMetrics(ctx, parsedMetric)
			if consumeErr != nil {
				c.logger.Error("Error consuming metrics", zap.Error(consumeErr))
				allErrs[j] = consumeErr
				continue
			}
		}

		// Join all the errors into a single error
		combinedErr := errors.Join(allErrs...)
		if combinedErr != nil {
			return fmt.Errorf("Error processing log record: %w", combinedErr)
		}
		return nil
	}

	// Should be impossible to reach this point
	return nil
}
