extern crate napi;
#[macro_use]
extern crate napi_derive;

pub mod error;
pub mod watcher;

use napi::{
    CallContext, Env, JsFunction, JsNull, JsObject, JsString, JsUndefined, Property, Ref, Result,
};
use watcher::{ParcelWatcher, ParcelWatcherEvent};

#[js_function(3)]
fn subscribe(ctx: CallContext) -> Result<JsNull> {
    let directory_to_watch = ctx.get::<JsString>(0);
    let js_callback_fn = ctx.get::<JsFunction>(1)?;
    let js_watch_options = ctx.get::<JsObject>(2)?;

    let this: JsObject = ctx.this_unchecked();
    let parcel_watcher: &mut ParcelWatcher = ctx.env.unwrap(&this)?;
    // TODO: Figure out how to get a function to be available after this function ends...
    // There's threadsafefunction but apparently that doesn't exist
    // and just passing the function returns in a segmentation fault so pretty sure the function ref gets destroyed...
    // Kinda don't know how to handle this at this point... :( was so close to getting a Proof of Concept...
    let callback_fn_ref: Ref<JsFunction> = None;
    parcel_watcher.watch(directory_to_watch?.into_utf8()?.as_str()?, callback_fn_ref)?;

    return ctx.env.get_null();
}

#[js_function]
fn process_events(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let parcel_watcher: &mut ParcelWatcher = ctx.env.unwrap(&this)?;

    parcel_watcher.process_events();

    return ctx.env.get_undefined();
}

#[js_function]
fn watcher_class_contructor(ctx: CallContext) -> Result<JsUndefined> {
    let mut this: JsObject = ctx.this_unchecked();
    ctx.env.wrap(&mut this, ParcelWatcher::new()?)?;
    //   this.set_named_property("count", ctx.env.create_int32(count)?)?;
    return ctx.env.get_undefined();
}

#[module_exports]
fn init(mut exports: JsObject, env: Env) -> Result<()> {
    let subscribe_method = Property::new(&env, "subscribe")?.with_method(subscribe);
    let process_events_method = Property::new(&env, "process_events")?.with_method(process_events);
    let watcher_class = env.define_class(
        "ParcelWatcher",
        watcher_class_contructor,
        &[subscribe_method, process_events_method],
    )?;
    exports.set_named_property("ParcelWatcher", watcher_class)?;
    return Ok(());
}
