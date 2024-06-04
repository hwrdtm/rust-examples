# What

This example demonstrates how to publish OTEL logs, metrics and traces to a proxy server which writes this data to a local file. Every piece of data appended to this local file is then picked up by a locally running OTEL Collector service, and exported to various outputs (console, Google Cloud etc.)

In order to achieve this, a custom connector called `filteringrouter` is implemented and built into a custom docker image of the OTEL Collector.

## Pre-Requisities

Do these steps before running this example.

### Service Account

You will need to obtain a service account key with the following permissions:

- `roles/monitoring.metricWriter`
- `roles/cloudtrace.agent`
- `roles/logging.logWriter`

Paste this locally under the file name `service-account-key.json`.

### Build Custom OTEL Collector

_This was last tested against commit `bf7cd57eb40ee90ae71af1365c50ed68002a4b07`._

You will need to build your own image of the OTEL Collector (contrib version):

1. Clone the [open-telemetry/opentelemetry-collector-contrib](https://github.com/open-telemetry/opentelemetry-collector-contrib) repo at `../../../open-telemetry/opentelemetry-collector-contrib`.
2. Run the following commands to set up that repo:
  - `make -j2 gomoddownload`
  - `make install-tools`
  - `make -j2 goporto`
  - `make crosslink`
  - `make gotidy`
3. Copy the `builder-config.custom.yaml` in this directory into `cmd/otelcontribcol` of that repo.
4. Modify the `make genotelcontribcol` command in the `Makefile` to use this new custom config file, eg. `$(BUILDER) --skip-compilation --config cmd/otelcontribcol/builder-config.yaml --output-path cmd/otelcontribcol`.
5. Run `make docker-otelcontribcol` to build the docker image. You should now see a `otelcontribcol` tagged with `latest` when you run `docker images`.

## Getting Started

1. Start the OTEL Collector with `docker compose up`.
2. Start the proxy server with `sudo cargo run --bin proxy-server`.
3. Run the publish OTEL script with `sudo cargo run --bin publish-otel`. You should see logs in the OTEL Collector service.

## Test File Offsets

The OTEL Collector uses a storage extension to store the file offset (cursor). When the OTEL Collector restarts, it will resume reading the files from this file offset.

To test this feature,

1. Start the OTEL Collector.
2. Start the proxy server
3. Run the publish OTEL script. You should see the logs in the OTEL Collector.
4. Shut down the OTEL Collector with `docker compose down`.
5. Run the publish OTEL script **twice**.
6. Start the OTEL Collector. You should see the logs for both the logs that were appended to the log file while the OTEL Collector was offline.

## Custom Connector: `filteringrouter`

This is a custom connector built into the OTEL Collector. This connector can connect from OTEL logs to any of logs/metrics/traces. 

This connector works correctly in both of the following arrangements:
1. When coupled with a single filelog receiver in a fan-out arrangement, ie. one filelog receiver connects to one instance of this connector, which then connects to three subsequent pipelines: metrics, traces and logs.
2. When coupled with a single filelog receiver in a single-in-single-out arrangement, ie. one filelog receiver connects to one instance of this connector, which then connects to one subsequent pipeline, either a metrics, traces or logs pipeline.
  - This is also the current arrangement of this example, where we use three instances of the filelog receiver, and three instances of this connector - one for each of metrics, traces and logs - and each filelog receiver / connector pipes to a single pipeline.