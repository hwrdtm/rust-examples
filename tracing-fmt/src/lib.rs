use std::str::FromStr;

#[cfg(feature = "ignore-fields")]
use field_format::CustomFieldFormatter;
use tracing::Subscriber;
use tracing_subscriber::{
    fmt::{self, format::Format},
    layer::SubscriberExt,
    EnvFilter,
};

mod event_format;
mod field_format;

pub fn init_subscriber() -> Result<impl Subscriber, Box<dyn std::error::Error>> {
    let level_filter =
        EnvFilter::try_from_default_env().or_else(|_e| EnvFilter::from_str("debug"))?;
    println!("Using level filter: {}", level_filter);

    // let custom_formatter = Format::default()
    let custom_formatter = event_format::CustomEventFormatter::default()
        .with_line_number(true)
        .with_event_scope(false)
        .with_prefix_string(Some("PREFIX".to_string()));

    let fmt_layer = {
        #[cfg(feature = "ignore-fields")]
        {
            fmt::layer()
                .fmt_fields(CustomFieldFormatter)
                .event_format(custom_formatter)
        }

        #[cfg(not(feature = "ignore-fields"))]
        {
            fmt::layer().event_format(custom_formatter)
        }
    };

    return Ok(tracing_subscriber::registry()
        .with(level_filter)
        .with(fmt_layer));
}
