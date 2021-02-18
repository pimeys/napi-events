mod channel;
mod registry;

use std::sync::Arc;

use channel::EventChannel;
use napi::{CallContext, Env, JsObject, JsString, JsUndefined, Property};
use napi_derive::{js_function, module_exports};
use registry::EventRegistry;
use tokio::sync::{mpsc, Mutex};
use tracing_futures::WithSubscriber;
use tracing_subscriber::{
    layer::{Layered, SubscriberExt},
    Registry,
};

#[derive(Clone)]
pub struct EventsTest {
    receiver: Arc<Mutex<mpsc::Receiver<String>>>,
    subscriber: Layered<EventChannel, EventRegistry>,
}

impl EventsTest {
    pub fn new(level: &str) -> Self {
        let (sender, receiver) = mpsc::channel(10000);

        let mut layer = EventChannel::new(sender);
        layer.filter_level(level.parse().unwrap());

        let subscriber = EventRegistry::new(Registry::default()).with(layer);

        Self {
            receiver: Arc::new(Mutex::new(receiver)),
            subscriber,
        }
    }

    pub async fn do_something(&self) -> usize {
        self.count().with_subscriber(self.subscriber.clone()).await
    }

    pub async fn receive_event(&self) -> Option<String> {
        self.receiver.lock().await.recv().await
    }

    async fn count(&self) -> usize {
        tracing::trace!(query = "SELECT 1", item_type = "query");
        tracing::info!(query = "SELECT 2", item_type = "query");
        1 + 2
    }
}

#[js_function(1)]
fn constructor(ctx: CallContext) -> napi::Result<JsUndefined> {
    let level = ctx.get::<JsString>(0)?.into_utf8()?;

    let mut this: JsObject = ctx.this_unchecked();
    let engine = EventsTest::new(level.as_str()?);

    ctx.env.wrap(&mut this, engine)?;
    ctx.env.get_undefined()
}

#[js_function(0)]
fn produce_events(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let test: &EventsTest = ctx.env.unwrap(&this)?;
    let test: EventsTest = test.clone();

    ctx.env
        .execute_tokio_future(async move { Ok(test.do_something().await) }, |&mut env, _| {
            env.get_undefined()
        })
}

#[js_function(0)]
fn receive_event(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let test: &EventsTest = ctx.env.unwrap(&this)?;
    let test: EventsTest = test.clone();

    ctx.env.execute_tokio_future(
        async move { Ok(test.receive_event().await) },
        |&mut env, event| match event {
            Some(ref event) => env.create_string(&event),
            None => env.get_null().and_then(|null| null.coerce_to_string()),
        },
    )
}

#[module_exports]
pub fn init(mut exports: JsObject, env: Env) -> napi::Result<()> {
    let query_engine = env.define_class(
        "EventsTest",
        constructor,
        &[
            Property::new(&env, "produce_events")?.with_method(produce_events),
            Property::new(&env, "receive_event")?.with_method(receive_event),
        ],
    )?;

    exports.set_named_property("EventsTest", query_engine)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::EventsTest;

    #[tokio::test]
    async fn simple_test() {
        let test = EventsTest::new("debug");

        let answer = test.do_something().await;
        assert_eq!(3, answer);

        let event = test.receive_event().await;
        assert_eq!(Some("foo".into()), event);
    }
}
