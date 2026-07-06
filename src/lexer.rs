use thiserror::Error;

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character: {0} at line {1}, col {2}")]
    UnexpectedChar(char, usize, usize),
    #[error("Unterminated string at line {0}, col {1}")]
    UnterminatedString(usize, usize),
    #[error("Invalid number: {0} at line {1}, col {2}")]
    InvalidNumber(String, usize, usize),
    #[error("Unterminated interpolation at line {0}, col {1}")]
    UnterminatedInterpolation(usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Keyword(String),
    Identifier(String),
    String(String),
    InterpolatedString(String, Vec<String>),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,

    LBrace, RBrace,
    LBracket, RBracket,
    LParen, RParen,
    Comma, Dot, Colon, Semicolon,
    Arrow,
    At,
    Hash,

    Plus, Minus, Star, Slash, Percent,
    Equal, EqualEqual,
    Bang, BangEqual,
    Less, LessEqual,
    Greater, GreaterEqual,
    Ampersand, Pipe,

    Newline,
    Indent,
    Dedent,

    EOF,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    indent_stack: Vec<usize>,
    pending: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            indent_stack: vec![0],
            pending: Vec::new(),
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if let Some(ch) = c {
            self.pos += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' {
                self.advance();
                return;
            }
            self.advance();
        }
    }

    fn read_string(&mut self) -> Result<String, LexerError> {
        let mut s = String::new();
        while let Some(c) = self.advance() {
            match c {
                '"' => return Ok(s),
                '\\' => {
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('t') => s.push('\t'),
                        Some('r') => s.push('\r'),
                        Some('"') => s.push('"'),
                        Some('\\') => s.push('\\'),
                        Some('{') => s.push('{'),
                        Some(c) => s.push(c),
                        None => return Err(LexerError::UnterminatedString(self.line, self.col)),
                    }
                }
                '{' if self.peek() == Some('{') => {
                    self.advance();
                    s.push('{');
                }
                '}' if self.peek() == Some('}') => {
                    self.advance();
                    s.push('}');
                }
                _ => s.push(c),
            }
        }
        Err(LexerError::UnterminatedString(self.line, self.col))
    }

    fn read_interpolated_string(&mut self) -> Result<Token, LexerError> {
        let mut parts: Vec<String> = Vec::new();
        let mut current = String::new();

        loop {
            match self.advance() {
                Some('"') => {
                    parts.push(current.clone());
                    let full = parts.concat();
                    return Ok(Token::InterpolatedString(full, parts));
                }
                Some('{') if self.peek() == Some('{') => {
                    self.advance();
                    current.push('{');
                }
                Some('{') => {
                    parts.push(current.clone());
                    current.clear();
                    let mut depth = 1;
                    let mut expr = String::new();
                    loop {
                        match self.advance() {
                            Some('{') => depth += 1,
                            Some('}') => {
                                depth -= 1;
                                if depth == 0 {
                                    parts.push(expr);
                                    current.clear();
                                    break;
                                }
                            }
                            Some(c) => expr.push(c),
                            None => return Err(LexerError::UnterminatedInterpolation(self.line, self.col)),
                        }
                    }
                }
                Some('\\') => {
                    match self.advance() {
                        Some('n') => current.push('\n'),
                        Some('t') => current.push('\t'),
                        Some('"') => current.push('"'),
                        Some('{') => current.push('{'),
                        Some('}') => current.push('}'),
                        Some(c) => current.push(c),
                        None => return Err(LexerError::UnterminatedString(self.line, self.col)),
                    }
                }
                Some(c) => current.push(c),
                None => return Err(LexerError::UnterminatedString(self.line, self.col)),
            }
        }
    }

    fn read_number(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.advance();
            } else if c == '.' && !is_float {
                is_float = true;
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        if is_float {
            Token::Float(s.parse().unwrap_or(0.0))
        } else {
            Token::Integer(s.parse().unwrap_or(0))
        }
    }

    fn read_identifier(&mut self, first: char) -> Token {
        let mut s = String::new();
        s.push(first);
        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                s.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match s.as_str() {
            "true" => Token::Boolean(true),
            "false" => Token::Boolean(false),
            "null" => Token::Null,
            _ => {
                let keywords = [
                    "cmd", "on", "config", "let", "set", "if", "elif", "else", "for", "in",
                    "reply", "send", "embed", "row", "button", "modal", "input", "select", "option",
                    "schedule", "cron", "every", "once", "after", "at", "import", "use", "export",
                    "middleware", "try", "catch", "finally", "throw", "thread", "parallel", "await",
                    "menu", "sub", "param", "autocomplete", "db", "http", "json", "file", "cache",
                    "voice", "webhook", "paginate", "std", "env", "type", "test", "mock", "assert",
                    "cancel", "discord", "slash", "prefix", "description", "permission", "cooldown",
                    "timezone", "headers", "body", "components", "ephemeral", "inline", "required",
                    "style", "id", "url", "disabled", "emoji", "placeholder", "values", "label",
                    "title", "desc", "color", "field", "footer", "image", "thumbnail", "timestamp",
                    "choices", "default", "optional", "status", "intents", "threads", "http_timeout",
                    "http_retry", "db_url", "db_auto_migrate", "cache_ttl", "cache_max_size",
                    "log_level", "log_file", "log_format", "error_channel", "error_ephemeral",
                    "allowed_paths", "max_file_size", "token_env", "env_prefix", "per", "user",
                    "channel", "role", "member", "attachment", "string", "int", "float", "bool",
                    "play", "stop", "join", "leave", "skip", "volume", "queue",
                    "as", "short", "paragraph", "info", "warn", "error", "success",
                    "danger", "primary", "secondary", "link", "emit",
                    "fn", "enum", "table", "migration", "model", "primary", "key",
                    "references", "unique", "index", "while", "break", "continue",
                    "match", "variant", "method",
                ];
                if keywords.contains(&s.as_str()) {
                    Token::Keyword(s)
                } else {
                    Token::Identifier(s)
                }
            }
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            if self.peek() == Some('\n') {
                self.advance();
                if let Some(last) = tokens.last() {
                    if *last != Token::Newline {
                        tokens.push(Token::Newline);
                    }
                }
                continue;
            }

            match self.peek() {
                None => {
                    tokens.push(Token::EOF);
                    return Ok(tokens);
                }
                Some('#') => {
                    self.skip_line();
                    continue;
                }
                Some('"') => {
                    self.advance();
                    if self.peek() == Some('"') {
                        self.advance();
                        if self.peek() == Some('"') {
                            self.advance();
                            let mut s = String::new();
                            loop {
                                match self.advance() {
                                    Some('"') if self.peek() == Some('"') && self.peek_next() == Some('"') => {
                                        self.advance(); self.advance();
                                        tokens.push(Token::String(s));
                                        break;
                                    }
                                    Some(c) => s.push(c),
                                    None => return Err(LexerError::UnterminatedString(self.line, self.col)),
                                }
                            }
                        } else {
                            tokens.push(Token::String(String::new()));
                        }
                    } else {
                        let mut in_string_has_brace = false;
                        let mut j = self.pos;
                        while j < self.chars.len() && self.chars[j] != '"' {
                            if self.chars[j] == '{' && self.chars.get(j+1) != Some(&'{') {
                                in_string_has_brace = true;
                                break;
                            }
                            if self.chars[j] == '\\' { j += 1; }
                            j += 1;
                        }
                        if in_string_has_brace {
                            tokens.push(self.read_interpolated_string()?);
                        } else {
                            tokens.push(Token::String(self.read_string()?));
                        }
                    }
                }
                Some('{') => { self.advance(); tokens.push(Token::LBrace); }
                Some('}') => { self.advance(); tokens.push(Token::RBrace); }
                Some('[') => { self.advance(); tokens.push(Token::LBracket); }
                Some(']') => { self.advance(); tokens.push(Token::RBracket); }
                Some('(') => { self.advance(); tokens.push(Token::LParen); }
                Some(')') => { self.advance(); tokens.push(Token::RParen); }
                Some(',') => { self.advance(); tokens.push(Token::Comma); }
                Some('.') => { self.advance(); tokens.push(Token::Dot); }
                Some(':') => { self.advance(); tokens.push(Token::Colon); }
                Some(';') => { self.advance(); tokens.push(Token::Semicolon); }
                Some('@') => { self.advance(); tokens.push(Token::At); }
                Some('+') => { self.advance(); tokens.push(Token::Plus); }
                Some('-') => {
                    if self.peek_next() == Some('>') {
                        self.advance(); self.advance();
                        tokens.push(Token::Arrow);
                    } else {
                        self.advance();
                        tokens.push(Token::Minus);
                    }
                }
                Some('*') => { self.advance(); tokens.push(Token::Star); }
                Some('/') => { self.advance(); tokens.push(Token::Slash); }
                Some('%') => { self.advance(); tokens.push(Token::Percent); }
                Some('!') => {
                    if self.peek_next() == Some('=') {
                        self.advance(); self.advance();
                        tokens.push(Token::BangEqual);
                    } else {
                        self.advance();
                        tokens.push(Token::Bang);
                    }
                }
                Some('=') => {
                    if self.peek_next() == Some('=') {
                        self.advance(); self.advance();
                        tokens.push(Token::EqualEqual);
                    } else {
                        self.advance();
                        tokens.push(Token::Equal);
                    }
                }
                Some('<') => {
                    if self.peek_next() == Some('=') {
                        self.advance(); self.advance();
                        tokens.push(Token::LessEqual);
                    } else {
                        self.advance();
                        tokens.push(Token::Less);
                    }
                }
                Some('>') => {
                    if self.peek_next() == Some('=') {
                        self.advance(); self.advance();
                        tokens.push(Token::GreaterEqual);
                    } else {
                        self.advance();
                        tokens.push(Token::Greater);
                    }
                }
                Some('&') => { self.advance(); tokens.push(Token::Ampersand); }
                Some('|') => { self.advance(); tokens.push(Token::Pipe); }
                Some(c) if c.is_ascii_digit() => {
                    let c = self.advance().unwrap();
                    tokens.push(self.read_number(c));
                }
                Some(c) if c.is_alphabetic() || c == '_' => {
                    let c = self.advance().unwrap();
                    tokens.push(self.read_identifier(c));
                }
                Some(c) => {
                    return Err(LexerError::UnexpectedChar(c, self.line, self.col));
                }
            }
        }
    }

    fn measure_indent(&self) -> usize {
        let mut count = 0;
        let mut i = self.pos;
        while i < self.chars.len() {
            match self.chars[i] {
                ' ' => count += 1,
                '\t' => count += 4,
                _ => break,
            }
            i += 1;
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        lexer.tokenize().unwrap()
    }

    #[test]
    fn test_empty() {
        let tokens = tokenize("");
        assert_eq!(tokens, vec![Token::EOF]);
    }

    #[test]
    fn test_keywords() {
        let tokens = tokenize("cmd on config let set if else for in reply");
        assert!(tokens.iter().any(|t| matches!(t, Token::Keyword(k) if k == "cmd")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Keyword(k) if k == "let")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Keyword(k) if k == "reply")));
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize(r#""hello world""#);
        assert_eq!(tokens[0], Token::String("hello world".into()));
    }

    #[test]
    fn test_numbers() {
        let tokens = tokenize("42 3.14");
        assert_eq!(tokens[0], Token::Integer(42));
        assert_eq!(tokens[1], Token::Float(3.14));
    }

    #[test]
    fn test_bool_null() {
        let tokens = tokenize("true false null");
        assert_eq!(tokens[0], Token::Boolean(true));
        assert_eq!(tokens[1], Token::Boolean(false));
        assert_eq!(tokens[2], Token::Null);
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("+ - * / % == != < > <= >= && || !");
        assert!(tokens.contains(&Token::Plus));
        assert!(tokens.contains(&Token::Minus));
        assert!(tokens.contains(&Token::Star));
        assert!(tokens.contains(&Token::Slash));
        assert!(tokens.contains(&Token::EqualEqual));
        assert!(tokens.contains(&Token::BangEqual));
    }

    #[test]
    fn test_braces_brackets() {
        let tokens = tokenize("{ } [ ] ( )");
        assert_eq!(tokens[0], Token::LBrace);
        assert_eq!(tokens[1], Token::RBrace);
        assert_eq!(tokens[2], Token::LBracket);
        assert_eq!(tokens[3], Token::RBracket);
        assert_eq!(tokens[4], Token::LParen);
        assert_eq!(tokens[5], Token::RParen);
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("# isso é um comentário\ncmd");
        assert!(tokens.iter().any(|t| matches!(t, Token::Keyword(k) if k == "cmd")));
    }

    #[test]
    fn test_interpolated_string() {
        let tokens = tokenize(r#""olá {name} bem-vindo""#);
        match &tokens[0] {
            Token::InterpolatedString(_, _) => {}, // ok
            Token::String(s) => assert!(s.contains("olá")),
            _ => panic!("expected string or interpolated string token, got {:?}", tokens[0]),
        }
    }

    #[test]
    fn test_identifier_underscore() {
        let tokens = tokenize("my_var member_join");
        assert_eq!(tokens[0], Token::Identifier("my_var".into()));
        assert_eq!(tokens[1], Token::Identifier("member_join".into()));
    }

    #[test]
    fn test_comma_dot_colon() {
        let tokens = tokenize("a, b.c: d");
        assert_eq!(tokens[1], Token::Comma);
        assert_eq!(tokens[3], Token::Dot);
        assert_eq!(tokens[5], Token::Colon);
    }
}
