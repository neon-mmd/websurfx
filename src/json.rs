//! A small JSON reader and writer.
//!
//! This project no longer depends on `serde` or `serde_json`, so every JSON boundary — the
//! upstream Qwant and SepiaSearch APIs, the settings cookie, the uploaded settings file and
//! every JSON response — is handled here instead.
//!
//! The reader borrows out of its input: a string without escape sequences is handed back as a
//! `&str` slice of the original buffer and a number keeps its raw text, so parsing an upstream
//! response only allocates the small container that holds the document.

use std::borrow::Cow;
use std::fmt;

/// The deepest nesting the reader accepts, so that a hostile payload cannot exhaust the stack.
const MAX_DEPTH: usize = 64;

/// The error returned when a document is not valid JSON or does not have the expected shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    /// What went wrong.
    reason: Cow<'static, str>,
    /// Byte offset into the document, when the problem has a position.
    offset: Option<usize>,
}

impl JsonError {
    /// Builds a JSON error that points at a byte offset.
    fn at(reason: &'static str, offset: usize) -> Self {
        Self {
            reason: Cow::Borrowed(reason),
            offset: Some(offset),
        }
    }

    /// Builds a JSON error about the shape of a document rather than a single position.
    pub fn missing(reason: impl Into<Cow<'static, str>>) -> Self {
        Self {
            reason: reason.into(),
            offset: None,
        }
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "{} at byte {offset}", self.reason),
            None => f.write_str(&self.reason),
        }
    }
}

impl std::error::Error for JsonError {}

/// A parsed JSON value.
///
/// Strings are borrowed from the document they were parsed from unless they contained escape
/// sequences, in which case the decoded form is owned.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue<'a> {
    /// The `null` literal.
    Null,
    /// A `true` or `false` literal.
    Bool(bool),
    /// A number, kept as the raw text it was written as.
    Number(&'a str),
    /// A string.
    String(Cow<'a, str>),
    /// An array of values.
    Array(Vec<JsonValue<'a>>),
    /// An object, kept in document order and looked up linearly.
    Object(Vec<(Cow<'a, str>, JsonValue<'a>)>),
}

impl<'a> JsonValue<'a> {
    /// Looks up a member of an object.
    ///
    /// # Arguments
    ///
    /// * `key` - It takes the name of the member to look for.
    ///
    /// # Returns
    ///
    /// Returns a reference to the member on success, or [`None`] when this value is not an
    /// object or the object has no such member.
    pub fn get(&self, key: &str) -> Option<&JsonValue<'a>> {
        match self {
            JsonValue::Object(members) => members
                .iter()
                .find(|(name, _)| name == key)
                .map(|(_, value)| value),
            _ => None,
        }
    }

    /// Returns the text of a string value.
    ///
    /// # Returns
    ///
    /// Returns a reference to the string on success, or [`None`] for any other value.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the text of a string value, keeping the lifetime of the document.
    ///
    /// Unlike [`JsonValue::as_str`] this hands back a [`Cow`] that is independent of the
    /// borrow of the value itself, so it can be stored in a struct that outlives the parsed
    /// document. A borrowed string stays borrowed, so this is still free for payloads without
    /// escape sequences.
    ///
    /// # Returns
    ///
    /// Returns the string on success, or [`None`] for any other value.
    pub fn to_cow(&self) -> Option<Cow<'a, str>> {
        match self {
            JsonValue::String(text) => Some(text.clone()),
            _ => None,
        }
    }

    /// Returns the value of a number as a signed integer.
    ///
    /// # Returns
    ///
    /// Returns the number on success, or [`None`] when the value is not a number or does not
    /// fit into an [`i64`].
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(raw) => raw.parse().ok(),
            _ => None,
        }
    }

    /// Returns the value of a number as a byte-sized unsigned integer.
    ///
    /// # Returns
    ///
    /// Returns the number on success, or [`None`] when the value is not a number or does not
    /// fit into a [`u8`].
    pub fn as_u8(&self) -> Option<u8> {
        match self {
            JsonValue::Number(raw) => raw.parse().ok(),
            _ => None,
        }
    }

    /// Returns the elements of an array.
    ///
    /// # Returns
    ///
    /// Returns the elements on success, or [`None`] for any other value.
    pub fn as_array(&self) -> Option<&[JsonValue<'a>]> {
        match self {
            JsonValue::Array(values) => Some(values),
            _ => None,
        }
    }
}

/// A cursor walking the bytes of a document.
struct Parser<'a> {
    /// The document being parsed.
    input: &'a [u8],
    /// How far into the document the cursor is.
    position: usize,
}

impl<'a> Parser<'a> {
    /// Returns the whole document with the lifetime of the parse, so slices of it can escape.
    fn input(&self) -> &'a [u8] {
        self.input
    }

    /// Returns the byte under the cursor without consuming it.
    fn peek(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }

    /// Advances past every JSON insignificant byte.
    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    /// Consumes the expected byte or fails.
    fn expect(&mut self, expected: u8, reason: &'static str) -> Result<(), JsonError> {
        match self.peek() {
            Some(byte) if byte == expected => {
                self.position += 1;
                Ok(())
            }
            _ => Err(JsonError::at(reason, self.position)),
        }
    }

    /// Parses any JSON value, refusing to recurse deeper than [`MAX_DEPTH`].
    fn parse_value(&mut self, depth: usize) -> Result<JsonValue<'a>, JsonError> {
        if depth >= MAX_DEPTH {
            return Err(JsonError::at(
                "maximum nesting depth exceeded",
                self.position,
            ));
        }

        match self.peek() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') => self.parse_literal(b"true").map(|()| JsonValue::Bool(true)),
            Some(b'f') => self
                .parse_literal(b"false")
                .map(|()| JsonValue::Bool(false)),
            Some(b'n') => self.parse_literal(b"null").map(|()| JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonValue::Number),
            _ => Err(JsonError::at("unexpected character", self.position)),
        }
    }

    /// Parses an object, keeping its members in document order.
    fn parse_object(&mut self, depth: usize) -> Result<JsonValue<'a>, JsonError> {
        self.expect(b'{', "expected `{`")?;

        let mut members = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(JsonValue::Object(members));
        }

        loop {
            self.skip_whitespace();
            let name = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':', "expected `:` after a member name")?;
            self.skip_whitespace();
            let value = self.parse_value(depth + 1)?;
            members.push((name, value));

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(JsonValue::Object(members));
                }
                _ => {
                    return Err(JsonError::at(
                        "expected `,` or `}` in object",
                        self.position,
                    ));
                }
            }
        }
    }

    /// Parses an array.
    fn parse_array(&mut self, depth: usize) -> Result<JsonValue<'a>, JsonError> {
        self.expect(b'[', "expected `[`")?;

        let mut values = Vec::new();
        self.skip_whitespace();

        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(JsonValue::Array(values));
        }

        loop {
            self.skip_whitespace();
            values.push(self.parse_value(depth + 1)?);

            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(JsonValue::Array(values));
                }
                _ => return Err(JsonError::at("expected `,` or `]` in array", self.position)),
            }
        }
    }

    /// Parses a string, borrowing it when it holds no escape sequences.
    fn parse_string(&mut self) -> Result<Cow<'a, str>, JsonError> {
        self.expect(b'"', "expected `\"`")?;

        let content_start = self.position;
        while let Some(byte) = self.peek() {
            match byte {
                b'"' => {
                    let end = self.position;
                    self.position += 1;
                    return Ok(Cow::Borrowed(self.text(content_start, end)?));
                }
                b'\\' => return self.parse_escaped_string(content_start),
                0x00..=0x1F => {
                    return Err(JsonError::at(
                        "unescaped control character in string",
                        self.position,
                    ));
                }
                _ => self.position += 1,
            }
        }

        Err(JsonError::at("unterminated string", self.position))
    }

    /// Parses a string that is known to hold at least one escape sequence.
    fn parse_escaped_string(&mut self, content_start: usize) -> Result<Cow<'a, str>, JsonError> {
        let mut decoded = String::from(self.text(content_start, self.position)?);

        loop {
            match self.peek() {
                Some(b'"') => {
                    self.position += 1;
                    return Ok(Cow::Owned(decoded));
                }
                Some(b'\\') => {
                    self.position += 1;
                    decoded.push(self.parse_escape()?);
                }
                Some(0x00..=0x1F) => {
                    return Err(JsonError::at(
                        "unescaped control character in string",
                        self.position,
                    ));
                }
                Some(_) => {
                    let run_start = self.position;
                    while let Some(byte) = self.peek() {
                        if byte == b'"' || byte == b'\\' || byte < 0x20 {
                            break;
                        }
                        self.position += 1;
                    }
                    decoded.push_str(self.text(run_start, self.position)?);
                }
                None => return Err(JsonError::at("unterminated string", self.position)),
            }
        }
    }

    /// Parses the single character escape that follows a backslash.
    fn parse_escape(&mut self) -> Result<char, JsonError> {
        let escape_at = self.position;
        let escape = self
            .peek()
            .ok_or_else(|| JsonError::at("unterminated escape sequence", escape_at))?;
        self.position += 1;

        match escape {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{8}'),
            b'f' => Ok('\u{c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_unicode_escape(),
            _ => Err(JsonError::at("invalid escape sequence", escape_at)),
        }
    }

    /// Parses a `\uXXXX` escape, joining a surrogate pair when one is present.
    fn parse_unicode_escape(&mut self) -> Result<char, JsonError> {
        let escape_at = self.position;
        let code = self.parse_four_hex_digits()?;

        if (0xD800..0xDC00).contains(&code) {
            let pair_at = self.position;
            let is_low_surrogate = self.peek() == Some(b'\\')
                && self.input().get(self.position + 1).copied() == Some(b'u');

            if is_low_surrogate {
                self.position += 2;
                let low = self.parse_four_hex_digits()?;

                if (0xDC00..0xE000).contains(&low) {
                    let combined = 0x1_0000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                    return char::from_u32(combined)
                        .ok_or_else(|| JsonError::at("invalid unicode escape", escape_at));
                }

                return Err(JsonError::at("invalid low surrogate", pair_at));
            }

            return Err(JsonError::at("unpaired high surrogate", escape_at));
        }

        char::from_u32(code).ok_or_else(|| JsonError::at("invalid unicode escape", escape_at))
    }

    /// Parses exactly four hexadecimal digits.
    fn parse_four_hex_digits(&mut self) -> Result<u32, JsonError> {
        let escape_at = self.position;
        let mut code = 0_u32;

        for _ in 0..4 {
            let byte = self
                .peek()
                .ok_or_else(|| JsonError::at("truncated unicode escape", escape_at))?;
            let digit = match byte {
                b'0'..=b'9' => u32::from(byte - b'0'),
                b'a'..=b'f' => u32::from(byte - b'a') + 10,
                b'A'..=b'F' => u32::from(byte - b'A') + 10,
                _ => return Err(JsonError::at("invalid unicode escape", escape_at)),
            };

            code = (code << 4) | digit;
            self.position += 1;
        }

        Ok(code)
    }

    /// Parses a number and returns the raw text it was written as.
    fn parse_number(&mut self) -> Result<&'a str, JsonError> {
        let start = self.position;

        if self.peek() == Some(b'-') {
            self.position += 1;
        }

        match self.peek() {
            Some(b'0') => self.position += 1,
            Some(b'1'..=b'9') => self.consume_digits(),
            _ => return Err(JsonError::at("invalid number", start)),
        }

        if self.peek() == Some(b'.') {
            self.position += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::at("invalid number", start));
            }
            self.consume_digits();
        }

        if matches!(self.peek(), Some(b'e' | b'E')) {
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(JsonError::at("invalid number", start));
            }
            self.consume_digits();
        }

        self.text(start, self.position)
    }

    /// Advances past a run of decimal digits.
    fn consume_digits(&mut self) {
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
    }

    /// Parses one of the three bare literals.
    fn parse_literal(&mut self, literal: &[u8]) -> Result<(), JsonError> {
        let start = self.position;
        let end = start + literal.len();

        if self.input().get(start..end) == Some(literal) {
            self.position = end;
            Ok(())
        } else {
            Err(JsonError::at("invalid literal", start))
        }
    }

    /// Returns a checked `&str` view of a slice of the document.
    fn text(&self, start: usize, end: usize) -> Result<&'a str, JsonError> {
        std::str::from_utf8(&self.input()[start..end])
            .map_err(|_| JsonError::at("invalid utf-8", start))
    }
}

/// Parses a JSON document.
///
/// # Arguments
///
/// * `input` - It takes the raw bytes of the document.
///
/// # Error
///
/// Returns a [`JsonError`] when the document is not valid JSON or has trailing content.
pub fn parse(input: &[u8]) -> Result<JsonValue<'_>, JsonError> {
    let mut parser = Parser { input, position: 0 };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();

    if parser.position != input.len() {
        return Err(JsonError::at(
            "trailing characters after the document",
            parser.position,
        ));
    }

    Ok(value)
}

/// A minimal append-only JSON writer.
///
/// Separating commas are inserted automatically, so callers only decide where values start
/// and end.
#[derive(Debug, Default)]
pub struct JsonWriter {
    /// The JSON text written so far.
    output: String,
    /// Whether a comma has to be written before the next value or member name.
    needs_comma: bool,
}

impl JsonWriter {
    /// Creates an empty writer with room for `capacity` bytes reserved.
    ///
    /// # Arguments
    ///
    /// * `capacity` - It takes the number of bytes to reserve up front.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            output: String::with_capacity(capacity),
            needs_comma: false,
        }
    }

    /// Writes a comma when the previous token was a completed value.
    fn before_value(&mut self) {
        if self.needs_comma {
            self.output.push(',');
        }
    }

    /// Starts an object.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn begin_object(&mut self) -> &mut Self {
        self.before_value();
        self.output.push('{');
        self.needs_comma = false;
        self
    }

    /// Closes the object that is currently open.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn end_object(&mut self) -> &mut Self {
        self.output.push('}');
        self.needs_comma = true;
        self
    }

    /// Starts an array.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn begin_array(&mut self) -> &mut Self {
        self.before_value();
        self.output.push('[');
        self.needs_comma = false;
        self
    }

    /// Closes the array that is currently open.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn end_array(&mut self) -> &mut Self {
        self.output.push(']');
        self.needs_comma = true;
        self
    }

    /// Writes a member name, which the next value becomes the value of.
    ///
    /// # Arguments
    ///
    /// * `name` - It takes the member name to write.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn field(&mut self, name: &str) -> &mut Self {
        self.before_value();
        push_string(&mut self.output, name);
        self.output.push(':');
        self.needs_comma = false;
        self
    }

    /// Writes a string value.
    ///
    /// # Arguments
    ///
    /// * `value` - It takes the text to write, escaped as required.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn string(&mut self, value: &str) -> &mut Self {
        self.before_value();
        push_string(&mut self.output, value);
        self.needs_comma = true;
        self
    }

    /// Writes a boolean value.
    ///
    /// # Arguments
    ///
    /// * `value` - It takes the boolean to write.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn boolean(&mut self, value: bool) -> &mut Self {
        self.before_value();
        self.output.push_str(if value { "true" } else { "false" });
        self.needs_comma = true;
        self
    }

    /// Writes an unsigned integer value.
    ///
    /// # Arguments
    ///
    /// * `value` - It takes the integer to write.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn unsigned(&mut self, value: u64) -> &mut Self {
        self.before_value();
        push_unsigned(&mut self.output, value);
        self.needs_comma = true;
        self
    }

    /// Writes a floating point value, or `null` when it is not finite.
    ///
    /// # Arguments
    ///
    /// * `value` - It takes the float to write.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn float(&mut self, value: f32) -> &mut Self {
        self.before_value();

        if value.is_finite() {
            push_float(&mut self.output, value);
        } else {
            // JSON has no representation for NaN or the infinities.
            self.output.push_str("null");
        }

        self.needs_comma = true;
        self
    }

    /// Writes a `null` value.
    ///
    /// # Returns
    ///
    /// Returns the writer itself so calls can be chained.
    pub fn null(&mut self) -> &mut Self {
        self.before_value();
        self.output.push_str("null");
        self.needs_comma = true;
        self
    }

    /// Consumes the writer and returns the JSON text.
    pub fn finish(self) -> String {
        self.output
    }
}

/// A stack-allocated `fmt::Write` sink, so formatting never touches the heap.
struct StackBuffer {
    /// The rendered bytes.
    bytes: [u8; Self::CAPACITY],
    /// How many of `bytes` are in use.
    length: usize,
}

impl StackBuffer {
    /// The most characters `f32`'s shortest round-trip form can take: a sign, seventeen
    /// significant digits, a decimal point and an exponent.
    const CAPACITY: usize = 32;

    /// Returns the rendered text.
    fn as_str(&self) -> &str {
        // The buffer only ever receives what `f32`'s `Display` writes, which is ASCII, so
        // the check cannot fail; falling back to `"0"` keeps the accessor total.
        std::str::from_utf8(&self.bytes[..self.length]).unwrap_or("0")
    }
}

impl std::fmt::Write for StackBuffer {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        let end = self.length + text.len();

        if end > Self::CAPACITY {
            return Err(std::fmt::Error);
        }

        self.bytes[self.length..end].copy_from_slice(text.as_bytes());
        self.length = end;

        Ok(())
    }
}

/// Appends `value` to `output` in the shortest form that round-trips.
///
/// This is the same text `f32::to_string` produces, written through a stack buffer instead of
/// a fresh `String`, so rendering a result score allocates nothing.
fn push_float(output: &mut String, value: f32) {
    use std::fmt::Write;

    let mut buffer = StackBuffer {
        bytes: [0; StackBuffer::CAPACITY],
        length: 0,
    };

    // The write cannot overflow the buffer: the shortest round-trip form of an `f32` is far
    // shorter than its capacity. Ignoring the result keeps this branch-free on the hot path.
    let _ = write!(buffer, "{value}");

    output.push_str(buffer.as_str());
}

/// Appends the decimal digits of `value` to `output`.
///
/// The digits are rendered into a stack buffer, so writing an integer never allocates.
fn push_unsigned(output: &mut String, value: u64) {
    /// The most decimal digits a `u64` can need.
    const MAX_DIGITS: usize = 20;

    let mut digits = [0_u8; MAX_DIGITS];
    let mut index = MAX_DIGITS;
    let mut remaining = value;

    loop {
        index -= 1;
        digits[index] = b'0' + (remaining % 10) as u8;
        remaining /= 10;

        if remaining == 0 {
            break;
        }
    }

    // The buffer only ever holds ASCII digits, so the check cannot fail; falling back to
    // `0` keeps the function total without reaching for `unsafe`.
    match std::str::from_utf8(&digits[index..]) {
        Ok(text) => output.push_str(text),
        Err(_) => output.push('0'),
    }
}

/// Reports whether a byte has to be escaped inside a JSON string.
///
/// Anything at or above 0x20 is passed through untouched, which keeps the check a single
/// comparison in the common case and leaves multi-byte UTF-8 sequences intact: their
/// continuation bytes are all above 0x80 and so never match here.
#[inline]
fn needs_escape(byte: u8) -> bool {
    byte == b'"' || byte == b'\\' || byte < 0x20
}

/// Appends `value` to `output` as a quoted and escaped JSON string.
///
/// Runs of text that need no escaping are copied in one go, so a typical result title or
/// snippet costs one scan and one `push_str` rather than a per-character match.
fn push_string(output: &mut String, value: &str) {
    /// The lower case hexadecimal digits used by `\u00XX` escapes.
    const HEX: [u8; 16] = *b"0123456789abcdef";

    output.push('"');

    let bytes = value.as_bytes();
    let mut run = 0;
    // Where the current unflushed segment begins, i.e. just past the last escaped byte.
    let mut start = 0;

    while run < bytes.len() {
        let byte = bytes[run];

        if !needs_escape(byte) {
            run += 1;
            continue;
        }

        // The text between the last escape and this one goes out in one copy.
        if run > start
            && let Ok(text) = std::str::from_utf8(&bytes[start..run])
        {
            output.push_str(text);
        }

        match byte {
            b'"' => output.push_str("\\\""),
            b'\\' => output.push_str("\\\\"),
            0x8 => output.push_str("\\b"),
            0xC => output.push_str("\\f"),
            b'\n' => output.push_str("\\n"),
            b'\r' => output.push_str("\\r"),
            b'\t' => output.push_str("\\t"),
            control => {
                // Every other code point below 0x20 has no short escape form.
                output.push_str("\\u00");
                output.push(HEX[usize::from(control >> 4)] as char);
                output.push(HEX[usize::from(control & 0xF)] as char);
            }
        }

        start = run + 1;
        run += 1;
    }

    if run > start
        && let Ok(text) = std::str::from_utf8(&bytes[start..run])
    {
        output.push_str(text);
    }

    output.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Asserts that a string value borrows from the document instead of owning its text.
    fn assert_borrowed<'a>(value: &JsonValue<'a>, input: &'a [u8]) {
        let JsonValue::String(text) = value else {
            assert!(
                matches!(value, JsonValue::String(_)),
                "expected a string value"
            );
            return;
        };

        let start = text.as_ptr() as usize;

        assert!(
            matches!(text, Cow::Borrowed(_)),
            "an unescaped string must not be copied"
        );
        assert!(
            start >= input.as_ptr() as usize
                && start + text.len() <= input.as_ptr() as usize + input.len(),
            "a borrowed string must point into the document"
        );
    }

    #[test]
    fn parses_scalars() {
        let input = br#"{"n":null,"t":true,"f":false,"s":"text","i":-12,"u":7}"#;
        let document = parse(input).unwrap();

        assert_eq!(document.get("n"), Some(&JsonValue::Null));
        assert_eq!(document.get("t"), Some(&JsonValue::Bool(true)));
        assert_eq!(document.get("f"), Some(&JsonValue::Bool(false)));
        assert_eq!(document.get("s").and_then(JsonValue::as_str), Some("text"));
        assert_eq!(document.get("i").and_then(JsonValue::as_i64), Some(-12));
        assert_eq!(document.get("u").and_then(JsonValue::as_u8), Some(7));
        assert_eq!(document.get("missing"), None);
        assert_borrowed(document.get("s").unwrap(), input);
    }

    #[test]
    fn parses_every_number_form() {
        let input = br#"[0,-0,12,3.5,-0.25,1e3,1E+3,2.5e-2,1234567890,-7]"#;
        let numbers = parse(input).unwrap();
        let numbers = numbers.as_array().unwrap();

        let integers: Vec<_> = numbers
            .iter()
            .map(|value| value.as_i64().unwrap_or_default())
            .collect();
        assert_eq!(integers, vec![0, 0, 12, 0, 0, 0, 0, 0, 1234567890, -7]);

        let floats: Vec<_> = numbers
            .iter()
            .map(|value| match value {
                JsonValue::Number(raw) => raw.parse::<f64>().unwrap_or_default(),
                _ => 0.0,
            })
            .collect();
        assert_eq!(
            floats,
            vec![
                0.0,
                0.0,
                12.0,
                3.5,
                -0.25,
                1000.0,
                1000.0,
                0.025,
                1234567890.0,
                -7.0
            ]
        );

        let raw: Vec<_> = numbers
            .iter()
            .map(|value| match value {
                JsonValue::Number(raw) => *raw,
                other => {
                    assert!(
                        matches!(other, JsonValue::Number(_)),
                        "expected a number, got {other:?}"
                    );
                    ""
                }
            })
            .collect();
        assert_eq!(
            raw,
            vec![
                "0",
                "-0",
                "12",
                "3.5",
                "-0.25",
                "1e3",
                "1E+3",
                "2.5e-2",
                "1234567890",
                "-7"
            ]
        );
    }

    #[test]
    fn parses_nested_containers() {
        let input = br#"{"a":[],"b":{},"c":[[1,2],[3]],"d":[{"k":"v"}]}"#;
        let document = parse(input).unwrap();

        assert_eq!(
            document.get("a").and_then(JsonValue::as_array),
            Some(&[][..])
        );
        assert_eq!(document.get("b"), Some(&JsonValue::Object(Vec::new())));
        assert_eq!(
            document
                .get("c")
                .and_then(JsonValue::as_array)
                .and_then(|values| values.first())
                .and_then(JsonValue::as_array)
                .map(<[JsonValue<'_>]>::len),
            Some(2)
        );
        assert_eq!(
            document
                .get("d")
                .and_then(JsonValue::as_array)
                .map(<[JsonValue<'_>]>::len),
            Some(1)
        );
    }

    #[test]
    fn decodes_every_escape_sequence() {
        let input = br#""\"\\\/\b\f\n\r\t\u0041\u00e9\u20ac""#;
        let document = parse(input).unwrap();

        assert_eq!(
            document.as_str(),
            Some("\"\\/\u{8}\u{c}\n\r\tA\u{e9}\u{20ac}")
        );
    }

    #[test]
    fn decodes_surrogate_pairs() {
        assert_eq!(
            parse(br#""\ud83d\ude00""#).unwrap().as_str(),
            Some("\u{1f600}"),
            "a valid pair must decode to one code point"
        );
        assert_eq!(
            parse(br#""\ud83d\ude00\ud83c\udf11""#).unwrap().as_str(),
            Some("\u{1f600}\u{1f311}"),
            "each pair must decode to its own code point"
        );

        // A surrogate without its partner cannot be represented.
        assert!(parse(br#""\ud800""#).is_err());
        assert!(parse(br#""\ud83d\ude00\ud800""#).is_err());
        assert!(parse(br#""\ud83dA""#).is_err());
        assert!(parse(br#""\udc00\udc00""#).is_err());
    }

    #[test]
    fn falls_back_to_owned_text_for_escaped_strings() {
        let input = br#"{"escaped":"a\nb","plain":"abc"}"#;
        let document = parse(input).unwrap();

        assert!(matches!(
            document.get("escaped"),
            Some(JsonValue::String(Cow::Owned(_)))
        ));
        assert_borrowed(document.get("plain").unwrap(), input);
    }

    #[test]
    fn rejects_malformed_documents() {
        for input in [
            &b""[..],
            b"{",
            b"}",
            b"[1,]",
            b"{\"a\":1,}",
            b"{\"a\" 1}",
            b"{a:1}",
            b"tru",
            b"01",
            b"1.",
            b"1e",
            b"+1",
            b"\"unterminated",
            b"\"raw\ncontrol\"",
            b"{} extra",
            b"[1 2]",
            br#""\q""#,
            br#""\u00""#,
            br#""\udc00\udc00""#,
        ] {
            assert!(parse(input).is_err(), "{input:?} must be rejected");
        }
    }

    #[test]
    fn refuses_documents_nested_past_the_depth_limit() {
        // `MAX_DEPTH` nested containers are accepted, one more is not.
        let shallow = "[".repeat(MAX_DEPTH) + &"]".repeat(MAX_DEPTH);
        assert!(parse(shallow.as_bytes()).is_ok());

        let too_deep = "[".repeat(MAX_DEPTH + 1) + &"]".repeat(MAX_DEPTH + 1);
        assert!(parse(too_deep.as_bytes()).is_err());
    }

    #[test]
    fn writes_valid_json_for_every_shape() {
        let mut writer = JsonWriter::default();
        writer.begin_object();
        writer.field("text").string("a\"b\\c\nd\u{1}");
        writer.field("flag").boolean(true);
        writer.field("count").unsigned(42);
        writer.field("score").float(0.75);
        writer.field("missing").null();
        writer.field("list").begin_array();
        writer.begin_object();
        writer.field("nested").begin_array();
        writer.string("x");
        writer.string("y");
        writer.end_array();
        writer.end_object();
        writer.begin_array();
        writer.string("z");
        writer.end_array();
        writer.end_array();
        writer.field("emptyObject").begin_object();
        writer.end_object();
        writer.field("emptyArray").begin_array();
        writer.end_array();
        writer.end_object();

        let rendered = writer.finish();
        let expected = concat!(
            r#"{"text":"a\"b\\c\nd\u0001","flag":true,"count":42,"score":0.75,"missing":null,"#,
            r#""list":[{"nested":["x","y"]},["z"]],"emptyObject":{},"emptyArray":[]}"#
        );

        assert_eq!(rendered, expected);

        // The produced text has to parse back into the same shape.
        let document = parse(rendered.as_bytes()).unwrap();
        assert_eq!(
            document.get("text").and_then(JsonValue::as_str),
            Some("a\"b\\c\nd\u{1}")
        );
        assert_eq!(document.get("count").and_then(JsonValue::as_i64), Some(42));
        assert_eq!(
            document
                .get("list")
                .and_then(JsonValue::as_array)
                .map(<[JsonValue<'_>]>::len),
            Some(2)
        );
    }

    #[test]
    fn writes_integers_of_every_width() {
        for value in [0_u64, 7, 10, 99, 100, 65535, u64::MAX] {
            let mut writer = JsonWriter::default();
            writer.begin_array();
            writer.unsigned(value);
            writer.end_array();

            assert_eq!(writer.finish(), format!("[{value}]"));
        }
    }

    #[test]
    fn writes_non_finite_floats_as_null() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let mut writer = JsonWriter::default();
            writer.begin_object();
            writer.field("value").float(value);
            writer.end_object();

            let rendered = writer.finish();
            assert_eq!(rendered, r#"{"value":null}"#);
            assert!(parse(rendered.as_bytes()).is_ok());
        }
    }

    #[test]
    fn reports_shape_errors_without_a_position() {
        let error = JsonError::missing("missing field `theme`");

        assert_eq!(error.to_string(), "missing field `theme`");
        assert_eq!(JsonError::at("bad", 7).to_string(), "bad at byte 7");
    }
}
