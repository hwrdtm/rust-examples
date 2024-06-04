package filteringrouter

import (
	"context"
	"errors"
	"fmt"

	"go.opentelemetry.io/collector/component"
	"go.opentelemetry.io/collector/consumer"
	"go.opentelemetry.io/collector/pdata/plog"
	"go.uber.org/zap"
)

// schema for connector
type logsConnectorImp struct {
	config       Config
	logsConsumer consumer.Logs
	logger       *zap.Logger
	// Include these parameters if a specific implementation for the Start and Shutdown function are not needed
	component.StartFunc
	component.ShutdownFunc
}

func newLogsConnector(logger *zap.Logger, config component.Config) (*logsConnectorImp, error) {
	logger.Info("Building filteringrouter logs connector")
	cfg := config.(*Config)

	return &logsConnectorImp{
		config: *cfg,
		logger: logger,
	}, nil
}

// Capabilities implements the consumer interface.
func (c *logsConnectorImp) Capabilities() consumer.Capabilities {
	return consumer.Capabilities{MutatesData: false}
}

func (c *logsConnectorImp) ConsumeLogs(ctx context.Context, ld plog.Logs) error {
	// When reading resourceLogs from a log line, we assume that the resourceLogs
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

			// Do nothing if the log body string does not start with {"resourceLogs"
			if err := validateStringBeginsWith(logBodyStr, "{\"resourceLogs\":"); err != nil {
				c.logger.Debug("Log body string validation failed", zap.Error(err))
				allErrs[j] = nil
				continue
			}

			// Try to parse the log body as a plog.Logs
			unmarshaler := plog.JSONUnmarshaler{}
			logBodyBytes := []byte(logBodyStr)
			parsedLogs, unmarshalErr := unmarshaler.UnmarshalLogs(logBodyBytes)
			if unmarshalErr != nil {
				c.logger.Error("Error unmarshalling logs", zap.Error(unmarshalErr))
				allErrs[j] = unmarshalErr
				continue
			}

			// Pass the parsed log to the next consumer
			consumeErr := c.logsConsumer.ConsumeLogs(ctx, parsedLogs)
			if consumeErr != nil {
				c.logger.Error("Error consuming logs", zap.Error(consumeErr))
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
