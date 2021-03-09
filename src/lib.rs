extern crate napi;
#[macro_use]
extern crate napi_derive;
extern crate notify;

use napi::{CallContext, JsFunction, JsNull, JsObject, JsString, Result};
use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

#[js_function(3)]
fn subscribe(ctx: CallContext) -> Result<JsNull> {
    let directory_to_watch = ctx.get::<JsString>(0)?.into_utf8()?.as_str()?;
    let js_callback_fn = ctx.get::<JsFunction>(1)?;
    let js_watch_options = ctx.get::<JsObject>(2)?;

    // Create a channel to receive the events.
    let (tx, rx) = channel();

    // TODO: Figure out how to get notify::error into napi::error
    // Automatically select the best implementation for your platform.
    // You can also access each implementation directly e.g. INotifyWatcher.
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2))?;

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher.watch(directory_to_watch, RecursiveMode::Recursive)?;

    // This is a simple loop, but you may want to use more complex logic here,
    // for example to handle I/O.
    loop {
        match rx.recv() {
            Ok(event) => println!("{:?}", event),
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    // js_callback_fn.call(None, &[js_directory_to_watch.into_unknown(), js_watch_options.into_unknown()])?;

    // return ctx.env.get_null();
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
    exports.create_named_method("subscribe", subscribe)?;

    Ok(())
}
