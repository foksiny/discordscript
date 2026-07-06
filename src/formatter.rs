use crate::lexer::{Lexer, Token};

pub fn format_source(source: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| format!("Lexer error: {}", e))?;

    let mut out = String::new();
    let mut indent: usize = 0;
    let mut prev_tok: Option<&Token> = None;
    let mut after_lbrace = false;
    let mut in_middleware_list = false;

    for (i, token) in tokens.iter().enumerate() {
        let next = tokens.get(i + 1);

        match token {
            Token::Newline if i == 0 || i == tokens.len() - 1 => continue,
            Token::Newline => {
                if after_lbrace && matches!(next, Some(Token::Newline) | Some(Token::RBrace)) {
                    continue;
                }
                if matches!(prev_tok, Some(Token::Newline)) {
                    continue;
                }
                if !after_lbrace && !matches!(prev_tok, Some(Token::RBrace)) {
                    out.push('\n');
                } else if after_lbrace {
                    // skip
                } else {
                    out.push('\n');
                }
                continue;
            }
            Token::LBrace => {
                out.push_str(" {\n");
                indent += 1;
                after_lbrace = true;
                prev_tok = Some(token);
                continue;
            }
            Token::RBrace => {
                indent = indent.saturating_sub(1);
                if !matches!(prev_tok, Some(Token::Newline)) && !matches!(prev_tok, Some(Token::RBrace)) {
                    out.push('\n');
                }
                out.push_str(&"    ".repeat(indent));
                out.push('}');
                after_lbrace = false;
                if matches!(next, Some(Token::RBrace)) {
                    // let the above code handle the next brace
                }
                if matches!(prev_tok, Some(Token::RBrace)) {
                    // already on new line
                }
                prev_tok = Some(token);
                continue;
            }
            Token::LBracket => {
                in_middleware_list = true;
                out.push('[');
                prev_tok = Some(token);
                continue;
            }
            Token::RBracket => {
                in_middleware_list = false;
                out.push(']');
                prev_tok = Some(token);
                continue;
            }
            Token::Comma => {
                if in_middleware_list {
                    out.push_str(", ");
                } else {
                    out.push_str(", ");
                }
                prev_tok = Some(token);
                continue;
            }
            _ => {}
        }

        after_lbrace = false;

        if matches!(prev_tok, Some(Token::Newline) | None) && !first_token(&out) {
            out.push_str(&"    ".repeat(indent));
        }

        let needs_space = !out.is_empty()
            && !out.ends_with(' ')
            && !out.ends_with('\n')
            && !out.ends_with('(')
            && !out.ends_with('[')
            && !out.ends_with(',');

        match token {
            Token::Keyword(s) => {
                if needs_space && !in_middleware_list {
                    out.push(' ');
                }
                out.push_str(s);
            }
            Token::Identifier(s) => {
                if needs_space {
                    out.push(' ');
                }
                out.push_str(s);
            }
            Token::String(s) => {
                if needs_space && !in_middleware_list {
                    out.push(' ');
                }
                out.push('"');
                out.push_str(s);
                out.push('"');
            }
            Token::InterpolatedString(s, parts) => {
                if needs_space {
                    out.push(' ');
                }
                out.push('"');
                // Simple reconstruction: string with #{} placeholders
                let mut combined = s.clone();
                for part in parts {
                    if let Some(pos) = combined.find("{}") {
                        let before = &combined[..pos];
                        let after = &combined[pos + 2..];
                        combined = format!("{}{}{}", before, part, after);
                    }
                }
                out.push_str(&combined);
                out.push('"');
            }
            Token::Integer(n) => {
                if needs_space {
                    out.push(' ');
                }
                out.push_str(&n.to_string());
            }
            Token::Float(f) => {
                if needs_space {
                    out.push(' ');
                }
                out.push_str(&f.to_string());
            }
            Token::Boolean(b) => {
                if needs_space {
                    out.push(' ');
                }
                out.push_str(&b.to_string());
            }
            Token::Null => {
                if needs_space {
                    out.push(' ');
                }
                out.push_str("null");
            }
            Token::LParen => out.push('('),
            Token::RParen => out.push(')'),
            Token::LBrace | Token::RBrace | Token::LBracket | Token::RBracket | Token::Comma => {
                // handled above
            }
            Token::Dot => out.push('.'),
            Token::Colon => out.push_str(": "),
            Token::Semicolon => out.push_str(";\n"),
            Token::Arrow => out.push_str(" -> "),
            Token::At => out.push('@'),
            Token::Hash => out.push('#'),
            Token::Plus => out.push_str(" + "),
            Token::Minus => out.push_str(" - "),
            Token::Star => out.push('*'),
            Token::Slash => out.push_str(" / "),
            Token::Percent => out.push_str(" % "),
            Token::Equal => {
                if needs_space {
                    out.push(' ');
                }
                out.push('=');
                if !matches!(next, Some(Token::RBrace) | Some(Token::Newline) | None) {
                    out.push(' ');
                }
            }
            Token::EqualEqual => out.push_str(" == "),
            Token::Bang => out.push('!'),
            Token::BangEqual => out.push_str(" != "),
            Token::Less => out.push_str(" < "),
            Token::LessEqual => out.push_str(" <= "),
            Token::Greater => out.push_str(" > "),
            Token::GreaterEqual => out.push_str(" >= "),
            Token::Ampersand => out.push_str(" && "),
            Token::Pipe => out.push_str(" || "),
            Token::Newline => {
                // handled above
            }
            Token::Indent | Token::Dedent | Token::EOF => {}
        }

        if !matches!(token, Token::Newline | Token::Indent | Token::Dedent | Token::EOF) {
            prev_tok = Some(token);
        }
    }

    let cleaned: Vec<&str> = out
        .lines()
        .map(|line| line.trim_end())
        .collect();
    let result = cleaned.join("\n").trim().to_string() + "\n";

    Ok(result)
}

fn first_token(out: &str) -> bool {
    out.trim().is_empty()
}
