#[cfg(test)]
macro_rules! single_token_test {
    ($name: ident, $parser: ident($content: expr) => $kind: expr) => {
        #[test]
        pub fn $name() {
            let content = format!("{} other_content", $content);
            let output = $parser($crate::parser::Span::new(&content)).unwrap();
            assert_eq!(
                output.0,
                $crate::parser::Span {
                    offset: $content.len(),
                    line: 1,
                    fragment: " other_content",
                    extra: ()
                }
            );
            assert_eq!(output.1.unwrap().value, $kind);
        }
    };
    ($name: ident, $parser: ident($content: expr) err $known_diag: expr) => {
        #[test]
        pub fn $name() {
            let content = format!("{} other_content", $content);
            let diags = $parser($crate::parser::Span::new(&content))
                .unwrap()
                .1
                .unwrap_err();
            assert_eq!(diags, diag::known($known_diag));
        }
    };
}
