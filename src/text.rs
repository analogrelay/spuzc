//! Text management APIs

#[derive(Debug, PartialEq, Eq)]
pub enum TextError {
    OutOfBounds,
}

pub struct Document {
    text: String,
}

impl Document {
    pub fn new<S: Into<String>>(text: S) -> Document {
        Document { text: text.into() }
    }
}

#[derive(Copy, Clone)]
pub struct Location<'a> {
    pub offset: usize,
    pub column: usize,
    pub line: usize,
    pub document: &'a Document,
}

impl<'a> Location<'a> {
    pub fn new(document: &'a Document) -> Location<'a> {
        Location {
            offset: 0,
            column: 0,
            line: 0,
            document,
        }
    }

    pub fn offset(&self, text: &str) -> Location<'a> {
        let (offset, line, column) =
            text.chars()
                .fold((self.offset, self.line, self.column), |(o, l, c), chr| {
                    if chr == '\n' {
                        (o + 1, l + 1, 0)
                    } else {
                        (o + 1, l, c + 1)
                    }
                });
        Location {
            offset,
            column,
            line,
            document: self.document,
        }
    }

    fn tup(&self) -> (usize, usize, usize) {
        (self.offset, self.line, self.column)
    }
}

pub struct Located<'a, T> {
    pub value: T,
    pub location: Location<'a>,
}

pub type Span<'a> = Located<'a, &'a str>;

pub struct Window<'a> {
    document: &'a Document,
    start: Location<'a>,
    end: Location<'a>,
}

impl<'a> Window<'a> {
    pub fn new(document: &'a Document) -> Window<'a> {
        Window {
            document,
            start: Location::new(document),
            end: Location::new(document),
        }
    }

    pub fn content(&self) -> &str {
        &self.document.text[self.start.offset..self.end.offset]
    }

    pub fn len(&self) -> usize {
        self.end.offset - self.start.offset
    }

    pub fn advance(&mut self, count: usize) -> Result<(), TextError> {
        let new_end = self.end.offset + count;
        if new_end > self.document.text.len() {
            Err(TextError::OutOfBounds)
        } else {
            // Update end
            let new_text = &self.document.text[self.end.offset..new_end];
            self.end = self.end.offset(new_text);
            Ok(())
        }
    }

    pub fn start(&self) -> Location<'a> {
        self.start
    }

    pub fn end(&self) -> Location<'a> {
        self.end
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
        assert_eq!(win.start().tup(), (0, 0, 0));
        assert_eq!(win.end().tup(), (0, 0, 0));
    }

    #[test]
    pub fn window_advance() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);
        win.advance(4).unwrap();
        assert_eq!(win.content(), "this");
        assert_eq!(win.start().tup(), (0, 0, 0));
        assert_eq!(win.end().tup(), (4, 0, 4));
    }

    #[test]
    pub fn window_advance_new_lines() {
        let doc = Document::new("this\r\nis\na\rtest");
        let mut win = Window::new(&doc);
        assert_eq!(win.start().tup(), (0, 0, 0));

        win.advance(6).unwrap();
        assert_eq!(win.content(), "this\r\n");
        assert_eq!(win.end().tup(), (6, 1, 0));

        win.advance(3).unwrap();
        assert_eq!(win.content(), "this\r\nis\n");
        assert_eq!(win.end().tup(), (9, 2, 0));

        win.advance(6).unwrap();
        assert_eq!(win.content(), "this\r\nis\na\rtest");
        assert_eq!(win.end().tup(), (15, 2, 6));
    }

    #[test]
    pub fn window_advance_out_of_bounds() {
        let doc = Document::new("this is a test document");
        let mut win = Window::new(&doc);
        assert_eq!(win.advance(64), Err(TextError::OutOfBounds));
    }
}
