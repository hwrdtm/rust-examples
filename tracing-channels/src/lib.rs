use std::{collections::HashMap, future::Future};

use flume::{
    r#async::{RecvFut, SendFut},
    Receiver, Sender,
};
use simple_observability_pipeline::opentelemetry::{
    global,
    propagation::{Extractor, Injector},
};
use tracing::{instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn new_bounded_channel<T>(cap: usize) -> (TracedSender<T>, TracedReceiver<T>) {
    let (tx, rx) = flume::bounded::<ChannelMsg<T>>(cap);
    (TracedSender { inner: tx }, TracedReceiver { inner: rx })
}

pub fn new_unbounded_channel<T>() -> (TracedSender<T>, TracedReceiver<T>) {
    let (tx, rx) = flume::unbounded::<ChannelMsg<T>>();
    (TracedSender { inner: tx }, TracedReceiver { inner: rx })
}

pub struct TracedSender<T> {
    inner: Sender<ChannelMsg<T>>,
}

impl<T> TracedSender<T> {
    #[instrument(level = "info", skip_all)]
    pub fn send_async(&self, data: T) -> SendFut<ChannelMsg<T>> {
        // Inject tracing context into metadata.
        let mut metadata = HashMap::new();
        let cx = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut ChannelMetadata(&mut metadata))
        });

        self.inner.send_async(ChannelMsg { metadata, data })
    }
}

pub struct TracedReceiver<T> {
    inner: Receiver<ChannelMsg<T>>,
}

impl<T> TracedReceiver<T> {
    /// Receive a value from the channel and return a span. If you wish to correlate all subsequent
    /// spans to be a child of this span, you MUST use the returned span to instrument
    /// all subsequent functions.
    ///
    /// The intention is to allow for establishing the following hierarchy of spans:
    /// - sender span
    ///   - recv span
    ///     - consumer span
    ///       - <work done in consumer>
    pub async fn recv_async(&self) -> <RecvFut<(ChannelMsg<T>, tracing::Span)> as Future>::Output {
        let recv_span = tracing::span!(tracing::Level::INFO, "recv_async");

        let mut msg = self
            .inner
            .recv_async()
            .instrument(recv_span.clone())
            .await?;

        // Extract the propagated tracing context from the incoming request headers.
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&ChannelMetadata(&mut msg.metadata))
        });

        // Set parent of recv_span to be the parent_cx that is propagated from the sender.
        recv_span.set_parent(parent_cx);

        // Create a new span for the work done in the receiver.
        let consumer_span = tracing::span!(tracing::Level::INFO, "consumer");

        // Set the parent of the consumer_span to be the recv_span.
        consumer_span.set_parent(recv_span.context());

        Ok((msg, consumer_span))
    }
}

pub struct ChannelMsg<T> {
    metadata: HashMap<String, String>,
    data: T,
}

pub struct ChannelMetadata<'a>(&'a mut HashMap<String, String>);

impl<'a> Injector for ChannelMetadata<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_lowercase(), value);
    }
}

impl<'a> Extractor for ChannelMetadata<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}
