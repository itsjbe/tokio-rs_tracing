#![cfg(feature = "env-filter")]

use tracing::{self, subscriber::with_default, Level};
use tracing_mock::{expect, subscriber};
use tracing_subscriber::{
    filter::EnvFilter,
    prelude::*,
};

/// Test that span field value filtering works correctly with integer values.
/// This tests the fix for field value filtering where directives with field values
/// should only enable events when the field values match.
#[test]
fn span_field_value_filtering_int() {
    // Filter: base level is info, but enable DEBUG for spans named "task" with field id=1
    let filter: EnvFilter = "info,[task{id=1}]=debug"
        .parse()
        .expect("filter should parse");

    let (subscriber, finished) = subscriber::mock()
        // First span with id=1
        .new_span(
            expect::span()
                .named("task")
                .at_level(Level::INFO)
                .with_fields(expect::field("id").with_value(&1u64)),
        )
        .enter(expect::span().named("task"))
        // This DEBUG event SHOULD be visible (inside span with id=1)
        .event(expect::event().at_level(Level::DEBUG))
        .exit(expect::span().named("task"))
        // Second span with id=2
        .new_span(
            expect::span()
                .named("task")
                .at_level(Level::INFO)
                .with_fields(expect::field("id").with_value(&2u64)),
        )
        .enter(expect::span().named("task"))
        // DEBUG event should NOT be visible here (wrong field value)
        // But INFO event should still be visible
        .event(expect::event().at_level(Level::INFO))
        .exit(expect::span().named("task"))
        .only()
        .run_with_handle();

    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        {
            let _span = tracing::info_span!("task", id = 1u64).entered();
            tracing::debug!("This should be visible (id=1)");
        }

        {
            let _span = tracing::info_span!("task", id = 2u64).entered();
            tracing::debug!("This should NOT be visible (id=2)");
            tracing::info!("This should be visible (base level is info)");
        }
    });

    finished.assert_finished();
}

/// Test that span field value filtering works correctly with string values.
/// This tests the original use case: [periodic_task{task="metrics_update"}]=debug
#[test]
fn span_field_value_filtering_string() {
    // Filter: base level is info, but enable DEBUG for spans named "periodic_task" with task=metrics_update
    // Note: We use unquoted strings in the filter since that's what the parser expects
    let filter: EnvFilter = "info,[periodic_task{task=metrics_update}]=debug"
        .parse()
        .expect("filter should parse");

    let (subscriber, finished) = subscriber::mock()
        // First span with task="metrics_update"
        .new_span(
            expect::span()
                .named("periodic_task")
                .at_level(Level::INFO)
                .with_fields(expect::field("task").with_value(&"metrics_update")),
        )
        .enter(expect::span().named("periodic_task"))
        // This DEBUG event SHOULD be visible (inside span with task="metrics_update")
        .event(expect::event().at_level(Level::DEBUG))
        .exit(expect::span().named("periodic_task"))
        // Second span with task="stale_node_cleanup"
        .new_span(
            expect::span()
                .named("periodic_task")
                .at_level(Level::INFO)
                .with_fields(expect::field("task").with_value(&"stale_node_cleanup")),
        )
        .enter(expect::span().named("periodic_task"))
        // DEBUG event should NOT be visible here (wrong field value)
        // But INFO event should still be visible
        .event(expect::event().at_level(Level::INFO))
        .exit(expect::span().named("periodic_task"))
        .only()
        .run_with_handle();

    let subscriber = subscriber.with(filter);

    with_default(subscriber, || {
        {
            let _span = tracing::info_span!("periodic_task", task = "metrics_update").entered();
            tracing::debug!("This should be visible (task=metrics_update)");
        }

        {
            let _span = tracing::info_span!("periodic_task", task = "stale_node_cleanup").entered();
            tracing::debug!("This should NOT be visible (task=stale_node_cleanup)");
            tracing::info!("This should be visible (base level is info)");
        }
    });

    finished.assert_finished();
}