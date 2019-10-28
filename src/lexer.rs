use logos::internal::LexerInternal;
use logos::{Lexer, Logos, Source};

#[derive(Debug, PartialEq, Clone, Copy, Logos)]
pub enum Token {
    #[end]
    EndOfProgram,

    #[regex = "\"[^\"]*\""]
    Text,

    #[regex = "'(\\\\x[0-9a-fA-F]+|\\\\[0-7]+|\\\\(n|t|r|\\\\|\"|'))'"]
    Char,

    #[regex = "-?([0-9]*\\.[0-9]+|[0-9]+\\.[0-9]*)"]
    Float,

    #[regex = "-?0(x|X)[0-9a-fA-F]+"]
    Hex,

    #[regex = "-?[0-9]+"]
    Integer,

    #[regex = "(struct|bool|char|int|float|string|color|dice)_t"]
    Type,

    #[regex = "[a-zA-Z_]+"]
    Identifier,

    #[regex = "#[0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F][0-9a-fA-F]"]
    Color,

    #[token = "{"]
    BraceOpen,

    #[token = "}"]
    BraceClose,

    #[token = "="]
    Assign,

    #[token = ","]
    Comma,

    #[token = "["]
    BracketOpen,

    #[token = "]"]
    BracketClose,

    #[regex = "//[^\n]*"]
    #[token = "/*"]
    #[callback = "ignore_comments"]
    #[error]
    UnexpectedToken,
    UnexpectedEndOfProgram,
}

fn ignore_comments<'source, Src: Source<'source>>(lex: &mut Lexer<Token, Src>) {
    use logos::Slice;

    if lex.slice().as_bytes() == b"/*" {
        loop {
            match lex.read() {
                None => return lex.token = Token::UnexpectedEndOfProgram,
                Some(b'*') => {
                    if lex.read_at(1) == Some(b'/') {
                        lex.bump(2);
                        break;
                    }
                }
                _ => lex.bump(1),
            }
        }
    }

    lex.advance();
}

#[cfg(test)]
mod tests {
    use super::Token;
    use logos::Logos;

    #[test]
    fn char_hex() {
        let sut = Token::lexer("'\\x9F' ");

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\x9F'");
    }

    #[test]
    fn char_oct() {
        let sut = Token::lexer("'\\200' ");

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\200'");
    }

    #[test]
    fn char_special() {
        let mut sut = Token::lexer("'\\n' '\\t' '\\r' '\\\\' '\\\"' '\\''");

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\n'");

        sut.advance();

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\t'");

        sut.advance();

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\r'");

        sut.advance();

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\\\'");

        sut.advance();

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\\"'");

        sut.advance();

        assert_eq!(sut.token, Token::Char);
        assert_eq!(sut.slice(), "'\\''");

        sut.advance();
    }
}
