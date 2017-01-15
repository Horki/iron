//! Formatting helpers for the logger middleware.

use std::default::Default;
use std::str::Chars;
use std::iter::Peekable;

use self::FormatText::{Method, URI, Status, ResponseTime, RemoteAddr, RequestTime};

/// A formatting style for the `Logger`, consisting of multiple
/// `FormatUnit`s concatenated into one line.
#[derive(Clone)]
pub struct Format(pub Vec<FormatUnit>);

impl Default for Format {
    /// Return the default formatting style for the `Logger`:
    ///
    /// ```ignore
    /// {method} {uri} -> {status} ({response-time})
    /// // This will be written as: {method} {uri} -> {status} ({response-time})
    /// ```
    fn default() -> Format {
        Format::new("{method} {uri} {status} ({response-time})").unwrap()
    }
}

impl Format {
    /// Create a `Format` from a format string, which can contain the fields
    /// `{method}`, `{uri}`, `{status}`, `{response-time}`, `{ip-addr}` and
    /// `{request-time}`.
    ///
    /// Returns `None` if the format string syntax is incorrect.
    pub fn new(s: &str) -> Option<Format> {

        let parser = FormatParser::new(s.chars().peekable());

        let mut results = Vec::new();

        for unit in parser {
            match unit {
                Some(unit) => results.push(unit),
                None => return None
            }
        }

        Some(Format(results))
    }
}

struct FormatParser<'a> {
    // The characters of the format string.
    chars: Peekable<Chars<'a>>,

    // A reusable buffer for parsing style attributes.
    object_buffer: String,

    finished: bool
}

impl<'a> FormatParser<'a> {
    fn new(chars: Peekable<Chars>) -> FormatParser {
        FormatParser {
            chars: chars,

            // No attributes are longer than 14 characters, so we can avoid reallocating.
            object_buffer: String::with_capacity(14),

            finished: false
        }
    }
}

// Some(None) means there was a parse error and this FormatParser should be abandoned.
impl<'a> Iterator for FormatParser<'a> {
    type Item = Option<FormatUnit>;

    fn next(&mut self) -> Option<Option<FormatUnit>> {
        // If the parser has been cancelled or errored for some reason.
        if self.finished { return None }

        // Try to parse a new FormatUnit.
        match self.chars.next() {
            // Parse a recognized object.
            //
            // The allowed forms are:
            //   - {method}
            //   - {uri}
            //   - {status}
            //   - {response-time}
            //   - {ip-addr}
            //   - {request-time}
            Some('{') => {
                self.object_buffer.clear();

                let mut chr = self.chars.next();
                while chr != None {
                    match chr.unwrap() {
                        // Finished parsing, parse buffer.
                        '}' => break,
                        c => self.object_buffer.push(c.clone())
                    }

                    chr = self.chars.next();
                }

                let text = match self.object_buffer.as_ref() {
                    "method" => Method,
                    "uri" => URI,
                    "status" => Status,
                    "response-time" => ResponseTime,
                    "request-time" => RequestTime,
                    "ip-addr" => RemoteAddr,
                    _ => {
                        // Error, so mark as finished.
                        self.finished = true;
                        return Some(None);
                    }
                };

                Some(Some(FormatUnit {text: text}))
            },

            // Parse a regular string part of the format string.
            Some(c) => {
                let mut buffer = String::new();
                buffer.push(c);

                loop {
                    match self.chars.peek() {
                        // Done parsing.
                        Some(&'{') | None => {
                            return Some(Some(FormatUnit {text: FormatText::Str(buffer)}))
                        },

                        Some(_) => {
                            buffer.push(self.chars.next().unwrap())
                        }
                    }
                }
            },

            // Reached end of the format string.
            None => None
        }
    }
}

/// A string of text to be logged. This is either one of the data
/// fields supported by the `Logger`, or a custom `String`.
#[derive(Clone)]
#[doc(hidden)]
pub enum FormatText {
    Str(String),
    Method,
    URI,
    Status,
    ResponseTime,
    RemoteAddr,
    RequestTime
}

/// A `FormatText` with associated style information.
#[derive(Clone)]
#[doc(hidden)]
pub struct FormatUnit {
    pub text: FormatText,
}
