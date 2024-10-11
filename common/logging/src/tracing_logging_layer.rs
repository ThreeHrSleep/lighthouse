use chrono::prelude::*;
use std::io::Write;
use tracing::field::{Field, Value, Visit};
use tracing::Subscriber;
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
pub struct LoggingLayer {
    pub non_blocking_writer: NonBlocking,
    pub guard: WorkerGuard,
}

impl<S> Layer<S> for LoggingLayer
where
    S: Subscriber,
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<S>) {
        let meta = event.metadata();
        let log_level = meta.level();
        let timestamp = Local::now().format("%b %d %H:%M:%S%.3f").to_string();

        let mut writer = self.non_blocking_writer.clone();

        let mut visitor = LogMessageExtractor {
            message: String::new(),
            fields: Vec::new(),
        };

        event.record(&mut visitor);

        let level_str = match *log_level {
            tracing::Level::ERROR => "\x1b[31mERROR\x1b[0m",
            tracing::Level::WARN => "\x1b[33mWARN\x1b[0m",
            tracing::Level::INFO => "\x1b[32mINFO\x1b[0m",
            tracing::Level::DEBUG => "\x1b[34mDEBUG\x1b[0m",
            tracing::Level::TRACE => "\x1b[35mTRACE\x1b[0m",
        };

        let bold_start = "\x1b[1m";
        let bold_end = "\x1b[0m";

        let bold_message = format!("{}{}{}", bold_start, visitor.message, bold_end);
        let mut formatted_fields = String::new();
        for (i, (field_name, field_value)) in visitor.fields.iter().enumerate() {
            if i > 0 {
                formatted_fields.push(' ');
            }
            let formatted_field =
                format!("{}{}{}={}", bold_start, field_name, bold_end, field_value);
            formatted_fields.push_str(&formatted_field);
        }

        let mut full_message = bold_message;
        if !formatted_fields.is_empty() {
            full_message = format!("{}\t\t{}", full_message, formatted_fields);
        }

        let message = format!("{} {} {}\n", timestamp, level_str, full_message);

        if let Err(e) = writer.write_all(message.as_bytes()) {
            eprintln!("Failed to write log: {}", e);
        }
    }
}

struct LogMessageExtractor {
    message: String,
    fields: Vec<(String, String)>,
}

impl tracing_core::field::Visit for LogMessageExtractor {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }

    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
        } else {
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}
