//! `flash.desktop.Clipboard` native methods

use crate::avm2::Error;
use crate::avm2::activation::Activation;
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::string::AvmString;

/// Implements `Clipboard.getSystemText`, which returns the current text
/// content of the system clipboard (or an empty string).
pub fn get_system_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text = activation.context.ui.clipboard_content();
    Ok(AvmString::new_utf8(activation.gc(), text).into())
}

/// Implements `Clipboard.setSystemText`, which replaces the text content of
/// the system clipboard.
pub fn set_system_text<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text = args.get_string_non_null(activation, 0, "text")?;
    activation
        .context
        .ui
        .set_clipboard_content(text.to_string());
    Ok(Value::Undefined)
}
