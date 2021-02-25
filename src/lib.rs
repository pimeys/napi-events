mod channel;
mod registry;

use channel::EventChannel;
use napi::{
    threadsafe_function::{ThreadSafeCallContext, ThreadsafeFunction},
    CallContext, Env, JsFunction, JsObject, JsString, JsUndefined, Property, Result,
};
use napi_derive::{js_function, module_exports};
use registry::EventRegistry;
use tracing_futures::WithSubscriber;
use tracing_subscriber::{
    layer::{Layered, SubscriberExt},
    Registry,
};

pub struct EventsTest {
    js_callback: ThreadsafeFunction<String>,
    subscriber: Layered<EventChannel, EventRegistry>,
}

impl EventsTest {
    pub fn new(level: &str, tsfn: ThreadsafeFunction<String>) -> napi::Result<Self> {
        let mut layer = EventChannel::new(tsfn.try_clone()?);
        layer.filter_level(level.parse().unwrap());

        let subscriber = EventRegistry::new(Registry::default()).with(layer);

        Ok(Self {
            subscriber,
            js_callback: tsfn,
        })
    }

    pub async fn do_something(&self) -> Result<usize> {
        Ok(self.count().with_subscriber(self.subscriber.clone()).await)
    }

    async fn count(&self) -> usize {
        tracing::trace!(query = "SELECT 1", item_type = "query");
        tracing::info!(query = "SELECT 2", item_type = "query");
        1 + 2
    }
}

#[js_function(2)]
fn constructor(ctx: CallContext) -> napi::Result<JsUndefined> {
    let level = ctx.get::<JsString>(0)?.into_utf8()?;

    let callback = ctx.get::<JsFunction>(1)?;
    let tsfn = ctx
        .env
        .create_threadsafe_function(&callback, 0, |tsfn_ctx: ThreadSafeCallContext<String>| {
            tsfn_ctx
                .env
                .create_string_from_std(tsfn_ctx.value)
                .map(|js_string| vec![js_string])
        })?;

    let mut this: JsObject = ctx.this_unchecked();
    let engine = EventsTest::new(level.as_str()?, tsfn)?;

    ctx.env.wrap(&mut this, engine)?;
    ctx.env.get_undefined()
}

#[js_function(0)]
fn produce_events(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let this_ref = ctx.env.create_reference(this)?;
    let test = ctx.env.unwrap_from_ref::<EventsTest>(&this_ref)?;

    ctx.env.execute_tokio_future(test.do_something(), |&mut env, _| {
        this_ref.unref(env)?;
        env.get_undefined()
    })
}

#[module_exports]
pub fn init(mut exports: JsObject, env: Env) -> napi::Result<()> {
    let query_engine = env.define_class(
        "EventsTest",
        constructor,
        &[Property::new(&env, "produce_events")?.with_method(produce_events)],
    )?;

    exports.set_named_property("EventsTest", query_engine)?;

    Ok(())
}
