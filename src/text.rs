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

    fn reset(&self, text: &str) -> Span {
        Span::new(self.start, self.start).expand(text)
    }

    fn expand(&self, text: &str) -> Span {
        let (offset, line, column) = text.chars().fold(
            (self.end.offset, self.end.line, self.end.column),
            |(o, l, c), chr| {
                if chr == '\n' {
                    (o + 1, l + 1, 0)
                } else {
                    (o + 1, l, c + 1)
                }
            },
        );
        Span {
            start: self.start,
            end: Location {
                offset,
                column,
                line,
            },
        }
    }
}

pub trait PeekPredicate {
    fn peek_predicate(&self, win: &Window) -> Option<usize>;
}

impl<F: Fn(char) -> bool> PeekPredicate for F {
    fn peek_predicate(&self, win: &Window) -> Option<usize> {
        match win.next() {
            Some(c) if self(c) => Some(1),
            _ => None,
        }
    }
}

impl PeekPredicate for char {
    fn peek_predicate(&self, win: &Window) -> Option<usize> {
        match win.next() {
            Some(c) if c == *self => Some(1),
            _ => None,
        }
    }
}

impl PeekPredicate for &str {
    fn peek_predicate(&self, win: &Window) -> Option<usize> {
        let l = self.len();
        if win.lookahead(l) == *self {
            Some(l)
        } else {
            None
        }
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
        if self.span.end.offset + 1 > self.document.text.len() {
            Err(TextError::OutOfBounds)
        } else {
            // Compute character width
            let (c, new_end) = read_char(&self.document.text, self.span.end.offset)?;
            self.span = self
                .span
                .expand(&self.document.text[self.span.end.offset..new_end]);
            Ok(c)
        }
    }

    // Look ahead `count` **bytes** and return the resulting string if it's valid.
    pub fn lookahead(&self, count: usize) -> &str {
        let end = std::cmp::min(self.document.text.len(), self.span.end.offset + count);
        match self.document.text.get(self.span.end.offset..end) {
            Some(s) => s,
            None => "",
        }
    }

    pub fn peek<P: PeekPredicate>(&self, predicate: P) -> bool {
        match predicate.peek_predicate(self) {
            Some(_) => true,
            None => false,
        }
    }

    pub fn take_until<P: PeekPredicate>(&mut self, predicate: P) -> Result<usize, TextError> {
        let mut count = 0;
        while let None = predicate.peek_predicate(self) {
            count += 1;
            self.take()?;
        }
        Ok(count)
    }

    pub fn take_while<P: PeekPredicate>(&mut self, predicate: P) -> Result<usize, TextError> {
        Urgh, need to determine if we're working in chars, bytes or both...
    }

    pub fn next(&self) -> Option<char> {
        // TODO: Could definitely unify this with take.
        if self.span.end.offset + 1 > self.document.text.len() {
            None
        } else {
            // Compute character width
            let (c, _) = match read_char(&self.document.text, self.span.end.offset + 1) {
                Ok(x) => x,
                Err(_) => return None,
            };
            Some(c)
        }
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

    pub fn backtrack(&mut self, count: usize) -> Result<(), TextError> {
        let new_end = (self.span.end.offset as isize) - (count as isize);
        if new_end < (self.span.start.offset as isize) {
            Err(TextError::OutOfBounds)
        } else {
            // Update end
            let new_text = &self.document.text[self.span.start.offset..(new_end as usize)];
            self.span = self.span.reset(new_text);
            Ok(())
        }
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
    pub fn window_lookahead() {
        let doc = Document::new("this is a");
        let mut win = Window::new(&doc);

        win.take_many(4).unwrap();
        assert_eq!(win.lookahead(3), " is");
        win.take_many(3).unwrap();
        win.advance();

        assert_eq!(win.lookahead(40), " a");
    }

    #[test]
    pub fn window_peek() {
        let doc = Document::new("this is a");
        let mut win = Window::new(&doc);

        win.take_many(4).unwrap();
        assert!(!win.peek("is"));
        assert!(win.peek(" is"));

        win.take_many(3).unwrap();
        assert!(!win.peek("a test document"));
    }

    #[test]
    pub fn window_backtrack() {
        let doc = Document::new("th\nis");
        let mut win = Window::new(&doc);

        win.take_many(5).unwrap();
        win.backtrack(3).unwrap();
        assert_eq!(win.content(), "th");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (2, 0, 2));
    }

    #[test]
    pub fn window_backtrack_out_of_bounds() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        win.take_many(4).unwrap();
        assert_eq!(win.backtrack(8), Err(TextError::OutOfBounds));
    }

    #[test]
    pub fn window_take_while() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while(char::is_alphabetic), Ok(4));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until(' '), Ok(4));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_until_str() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_until(" is"), Ok(4));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_take_while_str() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);

        assert_eq!(win.take_while("this"), Ok(4));
        assert_eq!(win.content(), "this");
        assert_eq!(win.span().start.tup(), (0, 0, 0));
        assert_eq!(win.span().end.tup(), (4, 0, 4));
    }
}
