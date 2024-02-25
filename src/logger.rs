use tracing::{level_filters::LevelFilter, Subscriber};
use tracing_subscriber::{
    fmt::{
        self,
        format::{Format, PrettyFields},
    },
    prelude::__tracing_subscriber_SubscriberExt,
    registry::LookupSpan,
    util::SubscriberInitExt,
    Layer,
};

pub fn init() {
    tracing_subscriber::registry().with(stdout_layer()).init();
}

fn stdout_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    fmt::layer()
        .without_time()
        .event_format(Format::default().with_source_location(true).without_time())
        .fmt_fields(PrettyFields::new())
        .with_target(false)
        .with_filter(LevelFilter::DEBUG)
}
