use crate::avm2::Avm2;
use crate::avm2::Avm2StrRepresentable;
use crate::avm2::activation::Activation;
use crate::avm2::error::{Error, Error2004Type, make_error_2004, make_error_2008, make_error_2175};
use crate::avm2::function::FunctionArgs;
use crate::avm2::globals::flash::display::display_object::initialize_for_allocator;
use crate::avm2::object::{ContentElementObject, ElementData, VectorObject};
use crate::avm2::parameters::ParametersExt;
use crate::avm2::value::Value;
use crate::display_object::{DisplayObject, EditText, TDisplayObject, TextLine};
use crate::font::FontType;
use crate::fte::{TextBaselineValue, TextLineCreationResultValue, TextRotationValue};
use crate::html::{self, FormatSpans, TextSpan};
use crate::string::{WStr, WString, utils as string_utils};
use crate::tag_utils::SwfMovie;
use crate::{avm2_stub_getter, avm2_stub_setter};
use std::ops::Range;
use swf::Twips;

pub use crate::avm2::object::text_block_allocator;

pub fn get_apply_non_linear_font_scaling<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.apply_non_linear_font_scaling().into())
}

pub fn set_apply_non_linear_font_scaling<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "applyNonLinearFontScaling"
    );

    this.set_apply_non_linear_font_scaling(args.get_bool(0));

    Ok(Value::Undefined)
}

pub fn get_baseline_font_description<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .baseline_font_description()
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn set_baseline_font_description<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "baselineFontDescription"
    );

    let value = args
        .try_get_object(0)
        .map(|v| v.as_font_description_object().unwrap());

    this.set_baseline_font_description(value, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_baseline_font_size<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_font_size().into())
}

pub fn set_baseline_font_size<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_f64(0);
    if value < 0.0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    avm2_stub_setter!(
        activation,
        "flash.text.engine.TextBlock",
        "baselineFontSize"
    );

    this.set_baseline_font_size(value);

    Ok(Value::Undefined)
}

pub fn get_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.baseline_zero().as_avm2_str(activation).into())
}

pub fn set_baseline_zero<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let value = args.get_string_non_null(activation, 0, "baselineZero")?;
    let value = TextBaselineValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextBaselineValue::UseDominantBaseline))
        .ok_or_else(|| make_error_2008(activation, "baselineZero"))?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "baselineZero");

    this.set_baseline_zero(value);

    Ok(Value::Undefined)
}

pub fn get_bidi_level<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.bidi_level().into())
}

pub fn set_bidi_level<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let value = args.get_i32(0);
    if value < 0 {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "bidiLevel");

    this.set_bidi_level(value);

    Ok(Value::Undefined)
}

pub fn get_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.line_rotation().as_avm2_str(activation).into())
}

pub fn set_line_rotation<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let value = args.get_string_non_null(activation, 0, "lineRotation")?;
    let value = TextRotationValue::from_avm2_str(&value)
        .filter(|v| !matches!(v, TextRotationValue::Auto))
        .ok_or_else(|| make_error_2008(activation, "lineRotation"))?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "lineRotation");

    this.set_line_rotation(value);

    Ok(Value::Undefined)
}

pub fn get_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .tab_stops()
        .map(|v| Value::from(VectorObject::from_vector(v.storage().clone(), activation)))
        .unwrap_or(Value::Null))
}

pub fn set_tab_stops<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    let tab_stops = args
        .try_get_object(0)
        .and_then(|v| v.as_vector_object())
        .map(|v| VectorObject::from_vector(v.storage().clone(), activation));

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "tabStops");

    this.set_tab_stops(tab_stops, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_text_justifier<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_justifier()
        .map(Value::from)
        .unwrap_or(Value::Null))
}

pub fn set_text_justifier<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let justifier = args.get_object(activation, 0, "textJustifier")?;

    avm2_stub_setter!(activation, "flash.text.engine.TextBlock", "textJustifier");

    this.set_text_justifier(justifier, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_content<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this.content().map(Value::from).unwrap_or(Value::Null))
}

pub fn set_content<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let content = args
        .try_get_object(0)
        .map(|v| v.as_content_element_object().unwrap());

    // Flash forbids setting a `TextBlock`'s content to a user-defined subclass
    // of `ContentElement`
    let is_invalid = content.is_some_and(|c| {
        let data = c.element_data();
        matches!(&*data, ElementData::Invalid)
    });
    if is_invalid {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    }

    this.set_content(content, activation.gc());

    Ok(Value::Undefined)
}

pub fn get_text_line_creation_result<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();
    Ok(this
        .text_line_creation_result()
        .map(|v| v.as_avm2_str(activation).into())
        .unwrap_or(Value::Null))
}

pub fn get_first_invalid_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    _this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    avm2_stub_getter!(
        activation,
        "flash.text.engine.TextBlock",
        "firstInvalidLine"
    );

    Ok(Value::Null)
}

pub fn get_first_line<'gc>(
    _activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line = this
        .first_line()
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn get_last_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    _args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    avm2_stub_getter!(activation, "flash.text.engine.TextBlock", "lastLine");

    let line = this
        .lines()
        .last()
        .map(|l| l.object2().expect("Already created"))
        .map(Value::from);

    Ok(line.unwrap_or(Value::Null))
}

pub fn release_lines<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this = this.as_object().unwrap().as_text_block_object().unwrap();

    let line_1 = args
        .get_object(activation, 0, "firstLine")?
        .as_display_object()
        .and_then(|o| o.as_text_line())
        .expect("Guaranteed by AS signature");

    let line_2 = args
        .get_object(activation, 1, "lastLine")?
        .as_display_object()
        .and_then(|o| o.as_text_line())
        .expect("Guaranteed by AS signature");

    // Flash has some unexpected behavior for certain edge cases of this method:

    // 1. Callers can swap the order of parameters to this method without ever
    //    affecting the functionality of the method.
    // 2. If two text lines are in the same text block and are siblings to each
    //    other, this method can still fail with an error if they can't be
    //    reached by iterating forward on the linked list of lines of this text
    //    block. It's possible to enter this scenario by rebreaking a line in
    //    the middle of a block.
    // 3. This method will always throw an error if either line is from a
    //    different text block.

    // The simplest way I could reproduce this behavior was with the following
    // logic:

    // 1. Start iterating over the linked list of lines, starting at
    //    `this.first_line()`.
    // 2. When reaching a line that is either `line_1` or `line_2` for the first
    //    time, record its position in the iterator.
    // 3. When reaching a line that is either `line_1` or `line_2` for the
    //    second time, record its position in the iterator and stop iterating.
    // 4. If either iteration reached the end of the list, return an error, as
    //    that means that one of the lines was not present in the block.
    // 5. Use the two positions to release the lines between `line_1` and
    //    `line_2` in the linked list of lines.

    let matches_either_line =
        |l| DisplayObject::ptr_eq(l, line_1) || DisplayObject::ptr_eq(l, line_2);

    let Some(first_position) = this.lines().position(matches_either_line) else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };

    if DisplayObject::ptr_eq(line_1, line_2) {
        // Special case: the two lines are the same. NOTE: We only do this after
        // we ensure that the line is actually reachable from `this.first_line`.
        line_1.release(activation.gc());

        return Ok(Value::Undefined);
    }

    let lines_count = this
        .lines()
        .skip(first_position + 1)
        .position(matches_either_line)
        .map(|p| p + 2);

    let Some(lines_count) = lines_count else {
        return Err(make_error_2004(activation, Error2004Type::ArgumentError));
    };

    let lines_to_remove = this
        .lines()
        .skip(first_position)
        .take(lines_count)
        .collect::<Vec<_>>();

    // `collect` the iterator, as we're modifying the doubly-linked list at the
    // same time as we iterate over it

    for line in lines_to_remove {
        line.release(activation.gc());
    }

    Ok(Value::Undefined)
}

pub fn do_create_text_line<'gc>(
    activation: &mut Activation<'_, 'gc>,
    this: Value<'gc>,
    args: FunctionArgs<'_, 'gc>,
) -> Result<Value<'gc>, Error<'gc>> {
    let this_obj = this.as_object().unwrap();
    let block = this_obj.as_text_block_object().unwrap();

    let line_to_use = args
        .try_get_object(0)
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_text_line());

    let previous_text_line = args
        .try_get_object(1)
        .and_then(|o| o.as_display_object())
        .and_then(|o| o.as_text_line());

    let width = args.get_f64(2);
    let fit_something = args.get_bool(4);

    let content = block.content().expect("Guaranteed by AS checks");

    let previous_position = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.end_index() as usize
    } else {
        0
    };

    let spans = match get_spans_from_content(content) {
        Ok(spans) => spans,
        Err(HandleContentError::NullElementFormat) => {
            // For some reason, FP handles this error as it handles an uncaught
            // exception, and returns `null` from this method.
            let error = make_error_2175(activation);

            Avm2::uncaught_error(activation, None, error, "Error creating TextLine");

            return Ok(Value::Null);
        }
    };

    let text = spans.text();

    if previous_position > text.len() {
        // This can happen when the content is changed after creating a TextLine.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
        return Ok(Value::Null);
    }

    let paragraph_end = next_line_break(text, previous_position);

    if text.is_empty() || paragraph_end == text.len() && previous_position == paragraph_end {
        // No more text.
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::Complete));
        return Ok(Value::Null);
    }

    // The text of the paragraph, without its separator.
    // If the content was changed after creating the previous line, this
    // line could start in the middle of a CRLF separator.
    let content_end =
        (paragraph_end - line_separator_length(&text[..paragraph_end])).max(previous_position);

    let movie = *activation.context.root_swf;
    let line_break = find_line_break(
        activation,
        movie,
        &spans,
        previous_position..content_end,
        width,
    );

    if !line_break.first_atom_fits && !fit_something {
        block.set_text_line_creation_result(Some(TextLineCreationResultValue::InsufficientWidth));
        return Ok(Value::Null);
    }

    let next_position = if line_break.position >= content_end {
        // The rest of the paragraph fits: the line includes the separator.
        paragraph_end
    } else {
        line_break.position
    };

    let line_index = if let Some(previous_text_line) = previous_text_line {
        previous_text_line.line_index() + 1
    } else {
        0
    };

    // `do_create_text_line` is called from both `TextBlock.recreateTextLine`
    // and `TextBlock.createTextLine`. The former passes this method an existing
    // `TextLine` to reuse, while the latter expects this method to create a new
    // `TextLine`.
    let text_line = if let Some(line) = line_to_use {
        // `TextLine.recreateTextLine` is the caller: completely reset the
        // properties of the passed line and use it.
        line.reset_properties(activation.gc());
        line
    } else {
        // `TextLine.createTextLine` is the caller: create a new `TextLine`.
        create_text_line(activation, width)
    };

    let text_line_instance = text_line.object2().expect("Already created the object2");

    let displayed_end = next_position.min(content_end);
    let line_spans = slice_spans(&spans, previous_position..displayed_end);
    text_line.set_content(line_spans, activation.context);

    text_line.set_text_block(Some(block), activation.gc());
    text_line.set_specified_width(width);
    text_line.set_raw_text_length((next_position - previous_position) as u32);
    text_line.set_begin_index(previous_position as u32);
    text_line.set_end_index(next_position as u32);
    text_line.set_line_index(line_index);

    if let Some(previous_line) = previous_text_line {
        text_line.set_previous_line(Some(previous_line), activation.gc());
        previous_line.set_next_line(Some(text_line), activation.gc());
    } else {
        // If there's no previous line, then this is the first line.
        block.set_first_line(Some(text_line), activation.gc());
    }

    // TODO correctly set the `validity`, `nextLine`, and `prevLine` properties
    // of the lines coming after this line

    let result = if line_break.is_emergency {
        TextLineCreationResultValue::Emergency
    } else {
        TextLineCreationResultValue::Success
    };
    block.set_text_line_creation_result(Some(result));

    Ok(text_line_instance.into())
}

fn create_text_line<'gc>(activation: &mut Activation<'_, 'gc>, width: f64) -> TextLine<'gc> {
    let class = activation.avm2().classes().textline;

    // TODO should we use the caller's movie instead? Does it matter?
    let movie = *activation.context.root_swf;

    let fallback = EditText::new_fte(activation.context, movie, 0.0, 0.0, width, 15.0);
    let text_line = TextLine::new(activation.context, movie, fallback);
    initialize_for_allocator(activation.context, text_line.into(), class);

    text_line
}

enum HandleContentError {
    NullElementFormat,
}

fn get_spans_from_content<'gc>(
    content: ContentElementObject<'gc>,
) -> Result<FormatSpans, HandleContentError> {
    let mut all_text = WString::new();
    let mut all_spans = Vec::new();

    handle_content_element(content, &mut all_text, &mut all_spans)?;

    let mut spans = FormatSpans::from_str_and_spans(&all_text, &all_spans);
    spans.normalize();
    Ok(spans)
}

fn handle_content_element<'gc>(
    content: ContentElementObject<'gc>,
    all_text: &mut WString,
    all_spans: &mut Vec<TextSpan>,
) -> Result<(), HandleContentError> {
    let data = content.element_data();
    let format = content.element_format();

    match &*data {
        ElementData::Text { text } => {
            // If `text` is `None`, FP just completely ignores the element. It
            // doesn't even check its `elementFormat`.
            if let Some(text) = text {
                all_text.push_str(text);

                let format = format.ok_or(HandleContentError::NullElementFormat)?;
                let text_format = format.as_text_format();

                let span = TextSpan::with_length_and_format(text.len(), &text_format);
                all_spans.push(span);
            }
        }
        ElementData::Group { elements } => {
            // TODO: The docs say GroupElement's format has some effects?
            for element in elements {
                handle_content_element(*element, all_text, all_spans)?;
            }
        }
        ElementData::Graphic => {
            // TODO
        }
        ElementData::Invalid => {
            unreachable!(
                "TextBlock and GroupElement prevent holding user subclasses of ContentElement"
            )
        }
    }

    Ok(())
}

fn next_line_break(text: &WStr, start: usize) -> usize {
    let remaining_text = &text[start..];
    let len = remaining_text
        .iter()
        .position(|ch| matches!(ch, 0x0A | 0x0D | 0x2028 | 0x2029))
        // Include the separator.
        .map(|pos| {
            if remaining_text.get(pos) == Some(0x0D) && remaining_text.get(pos + 1) == Some(0x0A) {
                pos + 2
            } else {
                pos + 1
            }
        });

    if let Some(len) = len {
        start + len
    } else {
        text.len()
    }
}

/// Returns the length of the line separator ending the given text, if any.
fn line_separator_length(text: &WStr) -> usize {
    let Some(last) = text.len().checked_sub(1).map(|i| text.at(i)) else {
        return 0;
    };

    match last {
        0x0A if text.len() >= 2 && text.at(text.len() - 2) == 0x0D => 2,
        0x0A | 0x0D | 0x2028 | 0x2029 => 1,
        _ => 0,
    }
}

/// The result of breaking a paragraph into a line.
struct LineBreak {
    /// The position at which the line ends. It is equal to the end of the
    /// paragraph when all of its remaining text fits on the line.
    position: usize,

    /// Whether no break opportunity could be found, and the line was broken
    /// in the middle of a word.
    is_emergency: bool,

    /// Whether the first atom of the line fits in the requested width.
    first_atom_fits: bool,
}

/// Find where the line laid out from the given range of the text, which is
/// a paragraph (or the end of one) without its separator, has to be broken
/// to fit in `width`.
///
/// This reuses the layout engine of `TextField`s, which breaks lines after
/// spaces and dashes, and otherwise in the middle of a word when nothing else
/// fits.
fn find_line_break<'gc>(
    activation: &mut Activation<'_, 'gc>,
    movie: SwfMovie<'gc>,
    spans: &FormatSpans,
    range: Range<usize>,
    width: f64,
) -> LineBreak {
    if range.is_empty() {
        return LineBreak {
            position: range.end,
            is_emergency: false,
            first_atom_fits: true,
        };
    }

    let text = spans.text();
    let paragraph = slice_spans(spans, range.clone());
    let width = Twips::from_pixels(width);
    let layout = html::lower_from_text_spans(
        &paragraph,
        activation.context,
        html::LayoutParams {
            movie,
            is_input: false,
            is_word_wrap: true,
            font_type: FontType::EmbeddedCFF,
        },
        Some(width),
    );

    let Some(first_line) = layout.lines().first() else {
        return LineBreak {
            position: range.end,
            is_emergency: false,
            first_atom_fits: true,
        };
    };

    // The layout engine may cut a surrogate pair in half when the first
    // character does not fit.
    let mut position = range.start + first_line.end().clamp(1, range.len());
    if position < range.end
        && matches!(text.at(position - 1), 0xD800..=0xDBFF)
        && matches!(text.at(position), 0xDC00..=0xDFFF)
    {
        position += 1;
    }

    let first_atom_length = if range.len() > 1
        && matches!(text.at(range.start), 0xD800..=0xDBFF)
        && matches!(text.at(range.start + 1), 0xDC00..=0xDFFF)
    {
        2
    } else {
        1
    };
    let first_atom_fits = first_line
        .char_x_bounds(first_atom_length - 1)
        .is_none_or(|(_, x_max)| x_max <= width);

    let is_emergency = position < range.end && {
        let previous = text.at(position - 1);
        let next = text.at(position);
        let is_opportunity = |c: u16| {
            char::from_u32(c.into())
                .is_some_and(|c| c.is_whitespace() || c == '-' || string_utils::swf_is_cjk_like(c))
        };
        !is_opportunity(previous) && !is_opportunity(next)
    };

    LineBreak {
        position,
        is_emergency,
        first_atom_fits,
    }
}

/// Extract the given range of the text and its formatting.
fn slice_spans(spans: &FormatSpans, range: Range<usize>) -> FormatSpans {
    let mut sliced_spans = Vec::new();
    let mut last_span = None;
    for (start, end, _text, span) in spans.iter_spans() {
        last_span = Some(span);
        if start > range.start && start >= range.end {
            break;
        }

        let length = end.min(range.end).saturating_sub(start.max(range.start));
        if length > 0 {
            let mut span = span.clone();
            span.span_length = length;
            sliced_spans.push(span);
        }
    }

    if sliced_spans.is_empty() {
        // An empty line still has the format of the text around it.
        let mut span = last_span.cloned().unwrap_or_default();
        span.span_length = 0;
        sliced_spans.push(span);
    }

    FormatSpans::from_str_and_spans(&spans.text()[range], &sliced_spans)
}
