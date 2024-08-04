use std::fmt::Debug;

use tracing_subscriber::{
    field::{RecordFields, VisitOutput},
    fmt::{
        format::{DefaultVisitor, Writer},
        FormatFields,
    },
};

/// A custom visitor that only records the field if it is the "message" field.
pub struct MessageVisitor<'a> {
    default_visitor: DefaultVisitor<'a>,
}

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.default_visitor.record_str(field, value);
    }

    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.default_visitor.record_error(field, value);
    }

    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn Debug) {
        // If the field is not "message", ignore that field.
        if field.name() != "message" {
            return;
        }

        self.default_visitor.record_debug(field, value);
    }
}

#[derive(Debug)]
pub struct CustomFieldFormatter;

impl FormatFields<'_> for CustomFieldFormatter {
    fn format_fields<R: RecordFields>(&self, writer: Writer, fields: R) -> std::fmt::Result {
        let default_visitor = DefaultVisitor::new(writer, true);
        let mut silent_visitor = MessageVisitor { default_visitor };
        fields.record(&mut silent_visitor);
        silent_visitor.default_visitor.finish()
    }
}
