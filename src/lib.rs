extern crate napi;
#[macro_use]
extern crate napi_derive;

use std::convert::TryInto;
use napi::{CallContext, JsNumber, JsString, JsNull, Either, JsObject, Result};
use std::path::Path;

#[js_function(0)]
fn find_first_file(ctx: CallContext) -> Result<Either<JsNull, JsString>> {
  let names = ctx.get::<JsObject>(0)?;
  let length: u32 = names.get_named_property::<JsNumber>("length")?.try_into()?;
  for i in 0..length {
    let n = names.get_element::<JsString>(i)?.into_utf8()?;
    let path = Path::new(n.as_str()?);

    if path.is_file() {
      return ctx.env.create_string(path.to_str().unwrap()).map(Either::B);
    }
  }

  return ctx.env.get_null().map(Either::A)
}

#[module_exports]
fn init(mut exports: JsObject) -> Result<()> {
  exports.create_named_method("findFirstFile", find_first_file)?;

  Ok(())
}