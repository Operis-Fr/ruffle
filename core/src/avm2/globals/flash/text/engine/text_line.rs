use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2006Type, make_error_2006, make_error_2008};
use crate::avm2::function::FunctionArgs;
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{TDisplayObject, TextLine};
use crate::fte::TextLineValidity;
use ruffle_macros::istr;
use std::ops::Range;
use swf::Point;

pub fn get_text_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    Ok(text_line.text_width().into())
}

pub fn get_has_tabs<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.has_tabs().into())
}

pub fn get_ascent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let text_line = display_object.as_text_line().unwrap();

    Ok(text_line.metrics().0.into())
}

pub fn get_descent<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let text_line = display_object.as_text_line().unwrap();

    Ok(text_line.metrics().1.into())
}

pub fn get_validity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(Value::Undefined);
    };

    let validity = match text_line.validity() {
        TextLineValidity::Valid => istr!("valid"),
        TextLineValidity::Invalid => istr!("invalid"),
        TextLineValidity::Static => istr!("static"),
        TextLineValidity::PossiblyInvalid => istr!("possiblyInvalid"),
        TextLineValidity::UserInvalid(string) => string,
    };

    Ok(validity.into())
}

pub fn set_validity<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(Value::Undefined);
    };

    let value = args.get_string_non_null(activation, 0, "validity")?;

    let previous_value = text_line.validity();
    let new_value = TextLineValidity::parse(value);

    let transition_allowed = match (previous_value, new_value) {
        (a, b) if a == b => true,
        (_, TextLineValidity::PossiblyInvalid) => false,
        (_, TextLineValidity::Static) => true,
        (TextLineValidity::Static, _) => false,
        (TextLineValidity::Invalid, _) => false,
        _ => true,
    };

    if !transition_allowed {
        return Err(make_error_2008(activation, "validity"));
    }

    text_line.set_validity(new_value, activation.gc());
    Ok(Value::Undefined)
}

pub fn get_text_block<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    let block = this.text_block_from_script();

    Ok(block.map(Value::from).unwrap_or(Value::Null))
}

pub fn get_specified_width<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.specified_width().into())
}

pub fn get_raw_text_length<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.raw_text_length().into())
}

pub fn get_text_block_begin_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this.begin_index().into())
}

pub fn get_previous_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this
        .previous_line()
        .and_then(|line| line.object2())
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn get_next_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this
        .as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap();

    Ok(this
        .next_line()
        .and_then(|line| line.object2())
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn get_text_height<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap();
    let display_object = this.as_display_object().unwrap();
    let Some(text_line) = display_object.as_text_line() else {
        return Ok(0.0.into());
    };

    Ok(text_line.text_height().into())
}

fn this_text_line<'gc>(this: Value<'gc>) -> TextLine<'gc> {
    this.as_object()
        .unwrap()
        .as_display_object()
        .unwrap()
        .as_text_line()
        .unwrap()
}

/// Returns the range of the atom at the given index, relative to the start of
/// the line, or throws a `RangeError` if there is no such atom.
fn atom_at<'gc>(
    activation: &mut Activation<'_, 'gc>,
    text_line: TextLine<'gc>,
    index: i32,
) -> Result<Range<usize>, Error<'gc>> {
    usize::try_from(index)
        .ok()
        .and_then(|index| text_line.atoms().get(index).cloned())
        .ok_or_else(|| make_error_2006(activation, Error2006Type::RangeError))
}

pub fn get_atom_count<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);

    Ok((text_line.atoms().len() as i32).into())
}

pub fn get_atom_index_at_char_index<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);
    let char_index = args.get_i32(0);

    let index = char_index
        .checked_sub(text_line.begin_index() as i32)
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| {
            text_line
                .atoms()
                .iter()
                .position(|atom| atom.contains(&index))
        })
        .map(|index| index as i32)
        .unwrap_or(-1);

    Ok(index.into())
}

pub fn get_atom_text_block_begin_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);
    let atom = atom_at(activation, text_line, args.get_i32(0))?;

    Ok((text_line.begin_index() as i32 + atom.start as i32).into())
}

pub fn get_atom_text_block_end_index<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);
    let atom = atom_at(activation, text_line, args.get_i32(0))?;

    Ok((text_line.begin_index() as i32 + atom.end as i32).into())
}

pub fn get_atom_bounds<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);
    let atom = atom_at(activation, text_line, args.get_i32(0))?;

    let (x_min, x_max) = text_line.atom_x_bounds(atom);
    let (ascent, descent) = text_line.metrics();

    activation.avm2().classes().rectangle.construct(
        activation,
        &[
            x_min.into(),
            (-ascent).into(),
            (x_max - x_min).into(),
            (ascent + descent).into(),
        ],
    )
}

pub fn get_atom_center<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);
    let atom = atom_at(activation, text_line, args.get_i32(0))?;

    let (x_min, x_max) = text_line.atom_x_bounds(atom);

    Ok(((x_min + x_max) / 2.0).into())
}

pub fn get_atom_word_boundary_on_left<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    #[derive(PartialEq)]
    enum CharClass {
        Space,
        Punctuation,
        Word,
    }

    fn char_class(c: u16) -> CharClass {
        match char::from_u32(c.into()) {
            Some(c) if c.is_whitespace() => CharClass::Space,
            Some(c) if c.is_ascii_punctuation() => CharClass::Punctuation,
            _ => CharClass::Word,
        }
    }

    let text_line = this_text_line(this);
    let atom = atom_at(activation, text_line, args.get_i32(0))?;

    if atom.start == 0 {
        return Ok(true.into());
    }

    // Line separators are not part of the text of the fallback text field,
    // but they are whitespace anyway.
    let text = text_line.fallback().text();
    let previous = text.get(atom.start - 1).map(char_class);
    let current = text.get(atom.start).map(char_class);

    let is_boundary = match (previous, current) {
        (Some(previous), Some(current)) => previous != current || current == CharClass::Punctuation,
        (Some(previous), None) => previous != CharClass::Space,
        (None, _) => false,
    };

    Ok(is_boundary.into())
}

pub fn get_atom_index_at_point<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let text_line = this_text_line(this);

    let global = Point::from_pixels(args.get_f64(0), args.get_f64(1));
    let Some(local) = text_line.global_to_local(global) else {
        return Ok((-1).into());
    };
    let (x, y) = (local.x.to_pixels(), local.y.to_pixels());

    let (ascent, descent) = text_line.metrics();
    if y < -ascent || y > descent {
        return Ok((-1).into());
    }

    let index = text_line
        .atoms()
        .into_iter()
        .map(|atom| text_line.atom_x_bounds(atom))
        .position(|(x_min, x_max)| x_min <= x && x < x_max)
        .map(|index| index as i32)
        .unwrap_or(-1);

    Ok(index.into())
}
