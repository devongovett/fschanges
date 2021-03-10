extern crate napi;
#[macro_use]
extern crate napi_derive;

pub mod error;
pub mod watcher;

use napi::{CallContext, JsFunction, JsNull, JsObject, JsString, JsUndefined, Property, Result};
use watcher::ParcelWatcher;

#[js_function]
fn watcher_class_contructor(ctx: CallContext) -> Result<JsUndefined> {
    let mut this: JsObject = ctx.this_unchecked();
    ctx.env.wrap(&mut this, ParcelWatcher::new()?)?;
    //   this.set_named_property("count", ctx.env.create_int32(count)?)?;
    return ctx.env.get_undefined();
}

#[js_function(1)]
fn new_watcher_class(ctx: CallContext) -> Result<JsFunction> {
    let subscribe_method = Property::new(&ctx.env, "subscribe")?.with_method(subscribe);
    let process_events_method = Property::new(&ctx.env, "process_events")?.with_method(process_events);
    let properties = vec![subscribe_method, process_events_method];
    return ctx.env.define_class(
        "ParcelWatcher",
        watcher_class_contructor,
        properties.as_slice(),
    );
}

#[js_function(3)]
fn subscribe(ctx: CallContext) -> Result<JsNull> {
    let directory_to_watch = ctx.get::<JsString>(0);
    let js_callback_fn = ctx.get::<JsFunction>(1)?;
    let js_watch_options = ctx.get::<JsObject>(2)?;

    let this: JsObject = ctx.this_unchecked();
    let parcel_watcher: &mut ParcelWatcher = ctx.env.unwrap(&this)?;

    parcel_watcher.watch(directory_to_watch?.into_utf8()?.as_str()?)?;

    return ctx.env.get_null();
}

#[js_function]
fn process_events(ctx: CallContext) -> Result<JsUndefined> {
    let this: JsObject = ctx.this_unchecked();
    let parcel_watcher: &mut ParcelWatcher = ctx.env.unwrap(&this)?;

    parcel_watcher.process_events();

    return ctx.env.get_undefined();
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("ParcelWatcher", watcher_class_contructor)?;
    return Ok(());
}
