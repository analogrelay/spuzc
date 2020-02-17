//! Text management APIs

// https://tools.ietf.org/html/rfc3629
static UTF8_CHAR_WIDTH: [u8; 256] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x1F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x3F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x5F
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, // 0x7F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0x9F
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, // 0xBF
    0, 0, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
    2, // 0xDF
    3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, // 0xEF
    4, 4, 4, 4, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // 0xFF
];

#[derive(Debug, PartialEq, Eq)]
pub enum TextError {
    InvalidUtf8Value(u8),
    OutOfBounds,
}

pub struct Document {
    pub text: String,
}

impl Document {
    pub fn new<S: Into<String>>(text: S) -> Document {
        Document { text: text.into() }
    }

    pub fn text_at(&self, sp: Span) -> &str {
        &self.text[sp.start.offset..sp.end.offset]
    }
}

#[derive(Copy, Clone)]
pub struct Location {
    pub offset: usize,
    pub column: usize,
    pub line: usize,
}

impl Location {
    pub const ZERO: Location = Location {
        offset: 0,
        column: 0,
        line: 0,
    };

    pub fn new(offset: usize, column: usize, line: usize) -> Location {
        Location {
            offset,
            column,
            line,
        }
    }

    pub(crate) fn tup(&self) -> (usize, usize, usize) {
        (self.offset, self.line, self.column)
    }
}

#[derive(Copy, Clone)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub const ZERO: Span = Span {
        start: Location::ZERO,
        end: Location::ZERO,
    };

    pub fn new(start: Location, end: Location) -> Span {
        Span { start, end }
    }

    fn expand(&self, c: char) -> Span {
        let (line, column) = match c {
            '\n' => (self.end.line + 1, 0),
            _ => (self.end.line, self.end.column + 1),
        };

        Span {
            start: self.start,
            end: Location {
                offset: self.end.offset + c.len_utf8(),
                column,
                line,
            },
        }
    }
}

pub trait CharPattern {
    fn matches(self, c: char) -> bool;
}

impl CharPattern for std::ops::Range<char> {
    fn matches(self, c: char) -> bool {
        self.contains(&c)
    }
}

impl CharPattern for std::ops::RangeInclusive<char> {
    fn matches(self, c: char) -> bool {
        self.contains(&c)
    }
}

impl CharPattern for char {
    fn matches(self, c: char) -> bool {
        self == c
    }
}

impl<F: Fn(char) -> bool> CharPattern for F {
    fn matches(self, c: char) -> bool {
        self(c)
    }
}

fn read_char(s: &str, at: usize) -> Result<(char, usize), TextError> {
    assert!(s.is_char_boundary(at));

    let next_byte = s.as_bytes()[at];
    let width = UTF8_CHAR_WIDTH[next_byte as usize];

    if width == 0 {
        Err(TextError::InvalidUtf8Value(next_byte))
    } else {
        let new_end = at + width as usize;
        let mut iter = s[at..new_end].chars();
        let c = iter.next().ok_or(TextError::OutOfBounds)?;
        assert!(iter.next().is_none());
        Ok((c, new_end))
    }
}

pub struct Window<'a> {
    document: &'a Document,
    span: Span,
}

impl<'a> Window<'a> {
    pub fn new(document: &'a Document) -> Window<'a> {
        Window {
            document,
            span: Span::ZERO,
        }
    }

    pub fn content(&self) -> &str {
        self.document.text_at(self.span)
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn take_many(&mut self, count: usize) -> Result<(), TextError> {
        for _ in 0..count {
            self.take()?;
        }
        Ok(())
    }

    pub fn take(&mut self) -> Result<char, TextError> {
        match self.next() {
            None => Err(TextError::OutOfBounds),
            Some(c) => {
                self.span = self.span.expand(c);
                Ok(c)
            }
        }
    }

    pub fn next(&self) -> Option<char> {
        if self.span.end.offset + 1 > self.document.text.len() {
            None
        } else {
            // Compute character width
            let (c, _) = match read_char(&self.document.text, self.span.end.offset) {
                Ok(x) => x,
                Err(_) => return None,
            };
            Some(c)
        }
    }

    pub fn peek<P: CharPattern>(&self, pattern: P) -> bool {
        match self.next() {
            None => false,
            Some(c) => pattern.matches(c),
        }
    }

    pub fn take_while<P: CharPattern + Clone>(&mut self, pattern: P) -> Result<(), TextError> {
        while self.peek(pattern.clone()) {
            self.take()?;
        }
        Ok(())
    }

    pub fn take_until<P: CharPattern + Clone>(&mut self, pattern: P) -> Result<(), TextError> {
        while !self.peek(pattern.clone()) {
            self.take()?;
        }
        Ok(())
    }

    pub fn advance(&mut self) -> Span {
        let ret = self.span;
        self.span = Span::new(self.span.end, self.span.end);
        ret
    }

    pub fn complete<T>(&mut self, value: T) -> Spanned<T> {
        let span = self.advance();
        Spanned::new(value, span)
    }
}

pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Spanned<T> {
        Spanned { value, span }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn empty_window() {
        let doc = Document::new("this is a test document");
        let win = Window::new(&doc);
        assert_eq!(win.content(), "");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (0, 0, 0));
    }

    #[test]
    pub fn window_take() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take(), Ok('t'));
        assert_eq!(win.take(), Ok('h'));
        assert_eq!(win.take(), Ok('i'));
        assert_eq!(win.take(), Ok('s'));

        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_emoji() {
        let doc = Document::new("✨a");
        let mut win = Window::new(&doc);

        assert_eq!(win.take(), Ok('✨'));
        assert_eq!(win.take(), Ok('a'));
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 2));
    }

    #[test]
    pub fn window_take_new_lines() {
        let doc = Document::new("this\r\nis\na\rtest");
        let mut win = Window::new(&doc);
        assert_eq!(win.span().start.tup(), (0, 0, 0));

        win.take_many(6).unwrap();
        assert_eq!(win.content(), "this\r\n");
        assert_eq!(win.span().end.tup(), (6, 1, 0));

        win.take_many(3).unwrap();
        assert_eq!(win.content(), "this\r\nis\n");
        assert_eq!(win.span().end.tup(), (9, 2, 0));

        win.take_many(6).unwrap();
        assert_eq!(win.content(), "this\r\nis\na\rtest");
        assert_eq!(win.span().end.tup(), (15, 2, 6));
    }

    #[test]
    pub fn window_take_out_of_bounds() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);
        assert_eq!(win.take_many(64), Err(TextError::OutOfBounds));
    }

    #[test]
    pub fn window_advance() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        win.take_many(4).unwrap();
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));

        win.advance();
        win.take_many(3).unwrap();
        assert_eq!(win.content(), " is");
        assert_eq!(win.span().start.tup(), (4, 0, 4));
        assert_eq!(win.span().end.tup(), (7, 0, 7));

        win.advance();
        win.take_many(2).unwrap();
        assert_eq!(win.content(), " a");
        assert_eq!(win.span().start.tup(), (7, 0, 7));
        assert_eq!(win.span().end.tup(), (9, 0, 9));
    }

    #[test]
    pub fn window_complete() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        win.take_many(4).unwrap();
        let ret = win.complete((42, "skidoo"));

        assert_eq!(win.content(), "");
        assert_eq!(win.span().start.tup(), (4, 0, 4));
        assert_eq!(win.span().end.tup(), (4, 0, 4));

        assert_eq!(ret.span.start.tup(), (0, 0, 0));
        assert_eq!(ret.span.end.tup(), (4, 0, 4));
        assert_eq!(ret.value, (42, "skidoo"));
    }

    #[test]
    pub fn window_peek() {
        let doc = Document::new("this is a");
        let mut win = Window::new(&doc);

        assert!(win.peek('t'));
        win.take_many(4).unwrap();
        assert!(!win.peek('i'));
        assert!(win.peek(char::is_whitespace));
    }

    #[test]
    pub fn window_peek_emoji() {
        let doc = Document::new("a✨a");
        let mut win = Window::new(&doc);

        assert!(!win.peek('✨'));
        win.take().unwrap();
        assert!(win.peek('✨'));
    }

    #[test]
    pub fn window_take_while() {
        let doc = Document::new("aaaab");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while('a'), Ok(()));
        assert_eq!(win.content(), "aaaa");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_while_range() {
        let doc = Document::new("1234test");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while('0'..='9'), Ok(()));
        assert_eq!(win.content(), "1234");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_while_emoji() {
        let doc = Document::new("✨✨✨✨b");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while('✨'), Ok(()));
        assert_eq!(win.content(), "✨✨✨✨");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (12, 0, 4));
    }

    #[test]
    pub fn window_take_while_fn() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while(char::is_alphabetic), Ok(()));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until(' '), Ok(()));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until_range() {
        let doc = Document::new("this42");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until('0'..='9'), Ok(()));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until_emoji() {
        let doc = Document::new("aaaa✨");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until('✨'), Ok(()));
        assert_eq!(win.content(), "aaaa");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until_fn() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until(char::is_whitespace), Ok(()));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }
}
