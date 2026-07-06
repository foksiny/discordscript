use crate::ast::*;
use crate::lexer::Token;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Unexpected token: {:?} at line {line}")]
    UnexpectedToken { token: Token, line: usize },
    #[error("Expected {expected}, got {:?}", found)]
    ExpectedToken { expected: String, found: Token, line: usize },
    #[error("{message} at line {line}")]
    Custom { message: String, line: usize },
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF)
    }

    fn peek_nth(&self, n: usize) -> Token {
        self.tokens.get(self.pos + n).cloned().unwrap_or(Token::EOF)
    }

    fn advance(&mut self) -> Token {
        let t = self.peek();
        self.pos += 1;
        t
    }

    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(&self.peek()) == std::mem::discriminant(expected)
    }

    fn expect(&mut self, expected: &Token, msg: &str) -> Result<Token, ParseError> {
        let t = self.peek();
        if std::mem::discriminant(&t) == std::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            Err(ParseError::ExpectedToken {
                expected: msg.to_string(),
                found: t,
                line: 0,
            })
        }
    }

    fn consume_newlines(&mut self) {
        while self.peek() == Token::Newline {
            self.advance();
        }
    }

    fn expect_keyword(&mut self, kw: &str) -> Result<(), ParseError> {
        match self.peek() {
            Token::Keyword(ref s) if s == kw => {
                self.advance();
                Ok(())
            }
            ref t => Err(ParseError::ExpectedToken {
                expected: format!("keyword '{}'", kw),
                found: t.clone(),
                line: 0,
            }),
        }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut program = Program {
            config: None,
            imports: Vec::new(),
            commands: Vec::new(),
            events: Vec::new(),
            schedules: Vec::new(),
            middleware: Vec::new(),
            context_menus: Vec::new(),
            types: Vec::new(),
            tests: Vec::new(),
            functions: Vec::new(),
            enums: Vec::new(),
            migrations: Vec::new(),
            models: Vec::new(),
        };

        self.consume_newlines();

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::EOF => break,
                Token::Keyword(ref kw) => {
                    match kw.as_str() {
                        "config" => {
                            self.advance();
                            program.config = Some(self.parse_config()?);
                        }
                        "import" => {
                            self.advance();
                            program.imports.push(self.parse_import()?);
                        }
                        "cmd" => {
                            self.advance();
                            program.commands.push(self.parse_command()?);
                        }
                        "on" => {
                            self.advance();
                            program.events.push(self.parse_event()?);
                        }
                        "schedule" => {
                            self.advance();
                            program.schedules.push(self.parse_schedule()?);
                        }
                        "middleware" => {
                            self.advance();
                            program.middleware.push(self.parse_middleware()?);
                        }
                        "menu" => {
                            self.advance();
                            program.context_menus.push(self.parse_context_menu()?);
                        }
                        "type" => {
                            self.advance();
                            program.types.push(self.parse_custom_type()?);
                        }
                        "test" => {
                            self.advance();
                            program.tests.push(self.parse_test()?);
                        }
                        "fn" => {
                            self.advance();
                            program.functions.push(self.parse_function_def()?);
                        }
                        "enum" => {
                            self.advance();
                            program.enums.push(self.parse_enum_def()?);
                        }
                        "table" | "migration" => {
                            self.advance();
                            program.migrations.push(self.parse_migration()?);
                        }
                        "model" => {
                            self.advance();
                            program.models.push(self.parse_model()?);
                        }
                        _ => {
                            return Err(ParseError::UnexpectedToken {
                                token: self.peek(),
                                line: 0,
                            });
                        }
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken {
                        token: self.peek(),
                        line: 0,
                    });
                }
            }
        }

        Ok(program)
    }

    fn parse_config(&mut self) -> Result<Config, ParseError> {
        self.expect(&Token::LBrace, "expected '{{' for config block")?;
        let mut config = Config {
            prefix: None,
            status: None,
            intents: Vec::new(),
            threads: None,
            http_timeout: None,
            http_retry: None,
            db_url: None,
            db_auto_migrate: None,
            cache_ttl: None,
            cache_max_size: None,
            log_level: None,
            log_file: None,
            log_format: None,
            error_channel: None,
            error_ephemeral: None,
            allowed_paths: Vec::new(),
            max_file_size: None,
            global_middleware: Vec::new(),
            token_env: None,
            env_prefix: None,
            db_models: Vec::new(),
        };

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated config block".into(), line: 0 }),
                Token::Keyword(ref kw) => {
                    let key = kw.clone();
                    self.advance();
                    match key.as_str() {
                        "prefix" => {
                            if let Token::String(s) = self.peek() { config.prefix = Some(s); self.advance(); }
                            else { config.prefix = Some("@".into()); }
                        }
                        "threads" => {
                            if let Token::Integer(n) = self.peek() { config.threads = Some(n); self.advance(); }
                        }
                        "http_timeout" => {
                            if let Token::Integer(n) = self.peek() { config.http_timeout = Some(n); self.advance(); }
                        }
                        "http_retry" => {
                            if let Token::Integer(n) = self.peek() { config.http_retry = Some(n); self.advance(); }
                        }
                        "db_url" => {
                            if let Token::String(s) = self.peek() { config.db_url = Some(s); self.advance(); }
                        }
                        "db_auto_migrate" => {
                            if let Token::Boolean(b) = self.peek() { config.db_auto_migrate = Some(b); self.advance(); }
                        }
                        "cache_ttl" => {
                            if let Token::Integer(n) = self.peek() { config.cache_ttl = Some(n); self.advance(); }
                        }
                        "cache_max_size" => {
                            if let Token::String(s) = self.peek() { config.cache_max_size = Some(s); self.advance(); }
                        }
                        "log_level" => {
                            if let Token::String(s) = self.peek() { config.log_level = Some(s); self.advance(); }
                        }
                        "log_file" => {
                            if let Token::String(s) = self.peek() { config.log_file = Some(s); self.advance(); }
                        }
                        "log_format" => {
                            if let Token::String(s) = self.peek() { config.log_format = Some(s); self.advance(); }
                        }
                        "error_channel" => {
                            if let Token::String(s) = self.peek() { config.error_channel = Some(s); self.advance(); }
                        }
                        "error_ephemeral" => {
                            if let Token::Boolean(b) = self.peek() { config.error_ephemeral = Some(b); self.advance(); }
                        }
                        "max_file_size" => {
                            if let Token::String(s) = self.peek() { config.max_file_size = Some(s); self.advance(); }
                        }
                        "token_env" => {
                            if let Token::String(s) = self.peek() { config.token_env = Some(s); self.advance(); }
                        }
                        "env_prefix" => {
                            if let Token::String(s) = self.peek() { config.env_prefix = Some(s); self.advance(); }
                        }
                        "status" => {
                            if let Token::String(t) = self.peek() { self.advance(); config.status = Some((t, String::new())); }
                            if let Token::String(s) = self.peek() { 
                                if let Some((ref mut t, _)) = config.status {
                                    *t = format!("{} {:?}", t, s);
                                }
                                self.advance();
                            }
                        }
                        "intents" => {
                            if self.peek() == Token::Identifier("all".to_string()) {
                                config.intents = vec!["all".into()];
                                self.advance();
                            } else if self.peek() == Token::LBracket {
                                self.advance();
                                loop {
                                    match self.peek() {
                                        Token::RBracket => { self.advance(); break; }
                                        Token::String(s) => { config.intents.push(s); self.advance(); }
                                        Token::Comma => { self.advance(); }
                                        _ => break,
                                    }
                                }
                            }
                        }
                        "allowed_paths" => {
                            if self.peek() == Token::LBracket {
                                self.advance();
                                loop {
                                    match self.peek() {
                                        Token::RBracket => { self.advance(); break; }
                                        Token::String(s) => { config.allowed_paths.push(s); self.advance(); }
                                        Token::Comma => { self.advance(); }
                                        _ => break,
                                    }
                                }
                            }
                        }
                        "middleware" => {
                            if self.peek() == Token::LBracket {
                                self.advance();
                                loop {
                                    match self.peek() {
                                        Token::RBracket => { self.advance(); break; }
                                        Token::String(s) => { config.global_middleware.push(s); self.advance(); }
                                        Token::Comma => { self.advance(); }
                                        _ => break,
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => { self.advance(); }
            }
        }

        Ok(config)
    }

    fn parse_import(&mut self) -> Result<Import, ParseError> {
        let path = match self.peek() {
            Token::String(s) => { self.advance(); s }
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "import path".into(), found: self.peek(), line: 0 }),
        };

        let mut alias = None;
        let mut symbols = Vec::new();

        if let Token::Keyword(ref k) = self.peek() {
            if k == "as" {
                self.advance();
                if let Token::Identifier(a) = self.peek() {
                    alias = Some(a);
                    self.advance();
                }
            }
        }

        if let Token::Keyword(ref k) = self.peek() {
            if k == "use" {
                self.advance();
                loop {
                    if let Token::Identifier(s) = self.peek() {
                        symbols.push(s);
                        self.advance();
                        if self.peek() == Token::Comma {
                            self.advance();
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        Ok(Import { path, alias, symbols })
    }

    fn parse_command(&mut self) -> Result<Command, ParseError> {
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::String(s) => { self.advance(); s }
            Token::Keyword(ref k) => { let s = k.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "command name".into(), found: self.peek(), line: 0 }),
        };

        self.expect(&Token::LBrace, "expected '{{' for command body")?;

        let mut cmd = Command {
            name,
            slash: false,
            prefix: None,
            description: None,
            guild_only: false,
            permission: None,
            cooldown: None,
            middleware: Vec::new(),
            params: Vec::new(),
            body: Vec::new(),
            subcommands: Vec::new(),
        };

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated command block".into(), line: 0 }),
                Token::Keyword(ref kw) => {
                    match kw.as_str() {
                        "slash" => {
                            self.advance();
                            if let Token::Boolean(b) = self.peek() {
                                cmd.slash = b;
                                self.advance();
                            } else {
                                cmd.slash = true;
                            }
                        }
                        "prefix" => {
                            self.advance();
                            if let Token::String(s) = self.peek() {
                                cmd.prefix = Some(s);
                                self.advance();
                            } else if self.peek() == Token::Boolean(true) {
                                cmd.prefix = Some("!".into());
                                self.advance();
                            } else {
                                cmd.prefix = Some("!".into());
                            }
                        }
                        "description" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { cmd.description = Some(s); self.advance(); }
                        }
                        "permission" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { cmd.permission = Some(s); self.advance(); }
                            else if let Token::Identifier(s) = self.peek() { cmd.permission = Some(s); self.advance(); }
                        }
                        "guild_only" => {
                            self.advance();
                            cmd.guild_only = true;
                        }
                        "cooldown" => {
                            self.advance();
                            let mut dur = 5;
                            let mut unit = "s".into();
                            let mut scope = None;
                            if let Token::Integer(n) = self.peek() { dur = n as i64; self.advance(); }
                            if let Token::String(s) = self.peek() { unit = s; self.advance(); }
                            if let Token::Keyword(ref k) = self.peek() {
                                if k == "per" {
                                    self.advance();
                                    if let Token::Identifier(s) = self.peek() { scope = Some(s); self.advance(); }
                                }
                            }
                            cmd.cooldown = Some(Cooldown { duration: dur, unit, scope });
                        }
                        "middleware" => {
                            self.advance();
                            if self.peek() == Token::LBracket {
                                self.advance();
                                loop {
                                    match self.peek() {
                                        Token::RBracket => { self.advance(); break; }
                                        Token::String(s) => { cmd.middleware.push(s); self.advance(); }
                                        Token::Comma => { self.advance(); }
                                        _ => break,
                                    }
                                }
                            } else if let Token::String(s) = self.peek() {
                                cmd.middleware.push(s);
                                self.advance();
                            }
                        }
                        "param" => {
                            self.advance();
                            cmd.params.push(self.parse_param()?);
                        }
                        "sub" => {
                            self.advance();
                            cmd.subcommands.push(self.parse_command()?);
                        }
                        _ => {
                            let stmt = self.parse_statement()?;
                            cmd.body.push(stmt);
                        }
                    }
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    cmd.body.push(stmt);
                }
            }
        }

        Ok(cmd)
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::Keyword(ref k) => { let s = k.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "parameter name".into(), found: self.peek(), line: 0 }),
        };

        let mut param = Param {
            name,
            param_type: TypeName::String,
            description: None,
            required: None,
            choices: None,
            autocomplete: false,
            default: None,
        };

        if self.peek() == Token::LBrace {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBrace => { self.advance(); break; }
                    Token::Keyword(ref kw) => {
                        match kw.as_str() {
                            "type" => {
                                self.advance();
                                if let Token::Identifier(t) = self.peek() {
                                    param.param_type = match t.as_str() {
                                        "string" => TypeName::String,
                                        "int" => TypeName::Int,
                                        "float" => TypeName::Float,
                                        "bool" => TypeName::Bool,
                                        "user" => TypeName::User,
                                        "channel" => TypeName::Channel,
                                        "role" => TypeName::Role,
                                        "member" => TypeName::Member,
                                        "attachment" => TypeName::Attachment,
                                        s => TypeName::Custom(s.into()),
                                    };
                                    self.advance();
                                }
                            }
                            "description" | "desc" => {
                                self.advance();
                                if let Token::String(s) = self.peek() { param.description = Some(s); self.advance(); }
                            }
                            "required" => {
                                self.advance();
                                if let Token::Boolean(b) = self.peek() { param.required = Some(b); self.advance(); }
                                else { param.required = Some(true); }
                            }
                            "choices" => {
                                self.advance();
                                if self.peek() == Token::LBracket {
                                    self.advance();
                                    let mut choices = Vec::new();
                                    loop {
                                        match self.peek() {
                                            Token::RBracket => { self.advance(); break; }
                                            Token::String(s) => { choices.push(s); self.advance(); }
                                            Token::Comma => { self.advance(); }
                                            _ => break,
                                        }
                                    }
                                    param.choices = Some(choices);
                                }
                            }
                            "autocomplete" => {
                                self.advance();
                                param.autocomplete = true;
                            }
                            "default" => {
                                self.advance();
                                match self.peek() {
                                    Token::String(s) => { param.default = Some(Literal::String(s, false)); self.advance(); }
                                    Token::Integer(n) => { param.default = Some(Literal::Integer(n)); self.advance(); }
                                    Token::Float(f) => { param.default = Some(Literal::Float(f)); self.advance(); }
                                    Token::Boolean(b) => { param.default = Some(Literal::Boolean(b)); self.advance(); }
                                    _ => {}
                                }
                            }
                            _ => { self.advance(); }
                        }
                    }
                    _ => { self.advance(); }
                }
            }
        }

        Ok(param)
    }

    fn parse_event(&mut self) -> Result<Event, ParseError> {
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::String(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "event name".into(), found: self.peek(), line: 0 }),
        };

        let filter = if let Token::String(s) = self.peek() {
            self.advance();
            Some(s)
        } else {
            None
        };

        self.expect(&Token::LBrace, "expected '{{' for event body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated event block".into(), line: 0 }),
                _ => {
                    let stmt = self.parse_statement()?;
                    body.push(stmt);
                }
            }
        }

        Ok(Event { name, filter, body })
    }

    fn parse_schedule(&mut self) -> Result<Schedule, ParseError> {
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::String(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "schedule name".into(), found: self.peek(), line: 0 }),
        };

        self.expect(&Token::LBrace, "expected '{{' for schedule body")?;

        let mut sched = Schedule {
            name,
            cron: None,
            every: None,
            every_unit: None,
            once_after: None,
            once_unit: None,
            at_time: None,
            timezone: None,
            body: Vec::new(),
        };

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated schedule block".into(), line: 0 }),
                Token::Keyword(ref kw) => {
                    match kw.as_str() {
                        "cron" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { sched.cron = Some(s); self.advance(); }
                        }
                        "every" => {
                            self.advance();
                            if let Token::Integer(n) = self.peek() { sched.every = Some(n); self.advance(); }
                            if let Token::String(s) = self.peek() { sched.every_unit = Some(s); self.advance(); }
                        }
                        "once" => {
                            self.advance();
                            if let Token::Keyword(ref k) = self.peek() {
                                if k == "after" {
                                    self.advance();
                                    if let Token::Integer(n) = self.peek() { sched.once_after = Some(n); self.advance(); }
                                    if let Token::String(s) = self.peek() { sched.once_unit = Some(s); self.advance(); }
                                }
                            }
                        }
                        "at" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { sched.at_time = Some(s); self.advance(); }
                        }
                        "timezone" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { sched.timezone = Some(s); self.advance(); }
                        }
                        _ => {
                            let stmt = self.parse_statement()?;
                            sched.body.push(stmt);
                        }
                    }
                }
                _ => {
                    let stmt = self.parse_statement()?;
                    sched.body.push(stmt);
                }
            }
        }

        Ok(sched)
    }

    fn parse_middleware(&mut self) -> Result<MiddlewareDef, ParseError> {
        let name = match self.peek() {
            Token::String(s) => { self.advance(); s }
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "middleware name".into(), found: self.peek(), line: 0 }),
        };

        self.expect(&Token::LBrace, "expected '{{' for middleware body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated middleware block".into(), line: 0 }),
                _ => {
                    let stmt = self.parse_statement()?;
                    body.push(stmt);
                }
            }
        }

        Ok(MiddlewareDef { name, body })
    }

    fn parse_context_menu(&mut self) -> Result<ContextMenu, ParseError> {
        let kind = match self.peek() {
            Token::Identifier(s) => { self.advance(); s }
            Token::Keyword(ref s) if s == "user" || s == "message" => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "context menu type (user/message)".into(), found: self.peek(), line: 0 }),
        };

        let name = match self.peek() {
            Token::String(s) => { self.advance(); s }
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "context menu name".into(), found: self.peek(), line: 0 }),
        };

        let mut permission = None;

        if let Token::Keyword(ref k) = self.peek() {
            if k == "permission" {
                self.advance();
                if let Token::String(s) = self.peek() { permission = Some(s); self.advance(); }
            }
        }

        self.expect(&Token::LBrace, "expected '{{' for context menu body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated context menu block".into(), line: 0 }),
                _ => {
                    let stmt = self.parse_statement()?;
                    body.push(stmt);
                }
            }
        }

        Ok(ContextMenu { kind, name, permission, body })
    }
    fn parse_custom_type(&mut self) -> Result<CustomType, ParseError> {
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedToken {
                expected: "type name".into(), found: self.peek(), line: 0,
            }),
        };

        let mut table_name = None;
        if let Token::String(s) = self.peek() {
            table_name = Some(s);
            self.advance();
        }

        self.expect(&Token::LBrace, "expected '{' for type definition")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom {
                    message: "Unterminated type block".into(), line: 0,
                }),
                Token::Keyword(ref k) if k == "fn" => {
                    self.advance();
                    methods.push(self.parse_function_def()?);
                }
                Token::Identifier(ref fname) => {
                    let fname = fname.clone();
                    self.advance();
                    let mut field_type = TypeName::String;
                    if let Ok(t) = self.parse_type_name() {
                        field_type = t;
                    }
                    let mut default = None;
                    let mut optional = false;
                    if let Token::Keyword(ref k) = self.peek() {
                        if k == "default" {
                            self.advance();
                            match self.peek() {
                                Token::String(s) => { default = Some(Literal::String(s, false)); self.advance(); }
                                Token::Integer(n) => { default = Some(Literal::Integer(n)); self.advance(); }
                                Token::Float(f) => { default = Some(Literal::Float(f)); self.advance(); }
                                Token::Boolean(b) => { default = Some(Literal::Boolean(b)); self.advance(); }
                                _ => {}
                            }
                        }
                    }
                    if let Token::Keyword(ref k) = self.peek() {
                        if k == "optional" {
                            optional = true;
                            self.advance();
                        }
                    }
                    fields.push(TypeField { name: fname, field_type, default, optional });
                }
                _ => { self.advance(); }
            }
        }

        Ok(CustomType { name, fields, methods, table_name })
    }

    fn parse_test(&mut self) -> Result<TestCase, ParseError> {
        let name = match self.peek() {
            Token::String(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "test name".into(), found: self.peek(), line: 0 }),
        };

        self.expect(&Token::LBrace, "expected '{{' for test body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated test block".into(), line: 0 }),
                _ => {
                    let stmt = self.parse_statement()?;
                    body.push(stmt);
                }
            }
        }

        Ok(TestCase { name, body })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume_newlines();

        match self.peek() {
            Token::Keyword(ref kw) => {
                match kw.as_str() {
                    "reply" => self.parse_reply(),
                    "send" => self.parse_send(),
                    "embed" => self.parse_embed_stmt(),
                    "row" => self.parse_row(),
                    "modal" => self.parse_modal(),
                    "let" => self.parse_let(),
                    "set" => self.parse_set(),
                    "if" => self.parse_if(),
                    "for" => self.parse_for(),
                    "thread" => self.parse_thread(),
                    "parallel" => self.parse_parallel(),
                    "await" => self.parse_await(),
                    "try" => self.parse_try_catch(),
                    "throw" => self.parse_throw(),
                    "cancel" => { self.advance(); Ok(Statement::Cancel) }
                    "return" => self.parse_return(),
                    "db" => self.parse_db(),
                    "http" => self.parse_http(),
                    "json" => self.parse_json(),
                    "file" => self.parse_file(),
                    "voice" => self.parse_voice(),
                    "webhook" => self.parse_webhook(),
                    "cache" => self.parse_cache(),
                    "paginate" => self.parse_paginate(),
                    "log" => self.parse_log(),
                    "std" => self.parse_std(),
                    "autocomplete" => self.parse_autocomplete(),
                    "mock" => self.parse_mock(),
                    "assert" => self.parse_assert(),
                    "discord" => self.parse_discord(),
                    "while" => self.parse_while(),
                    "break" => { self.advance(); Ok(Statement::Break) }
                    "continue" => { self.advance(); Ok(Statement::Continue) }
                    "match" => self.parse_match(),
                    _ => self.parse_expression_stmt(),
                }
            }
            Token::Identifier(_) => {
                self.parse_expression_stmt()
            }
            _ => {
                self.parse_expression_stmt()
            }
        }
    }

    fn parse_reply(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let content = self.parse_pattern()?;
        let mut ephemeral = None;
        let mut components = None;

        loop {
            match self.peek() {
                Token::Keyword(ref k) if k == "ephemeral" => {
                    self.advance();
                    if let Token::Boolean(b) = self.peek() { ephemeral = Some(b); self.advance(); }
                    else { ephemeral = Some(true); }
                }
                Token::Keyword(ref k) if k == "components" => {
                    self.advance();
                    if let Token::Boolean(b) = self.peek() { components = Some(b); self.advance(); }
                    else { components = Some(true); }
                }
                _ => break,
            }
        }

        Ok(Statement::Reply { content, ephemeral, components })
    }

    fn parse_send(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let content = self.parse_pattern()?;
        let mut channel = None;

        if let Token::Keyword(ref k) = self.peek() {
            if k == "in" {
                self.advance();
                if let Token::Keyword(ref k2) = self.peek() {
                    if k2 == "channel" {
                        self.advance();
                    }
                }
                channel = Some(self.parse_pattern()?);
            }
        }

        Ok(Statement::Send { content, channel })
    }

    fn parse_embed_stmt(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        Ok(Statement::Embed(self.parse_embed()?))
    }

    fn parse_embed(&mut self) -> Result<Embed, ParseError> {
        self.expect(&Token::LBrace, "expected '{{' for embed")?;

        let mut embed = Embed {
            title: None,
            description: None,
            color: None,
            fields: Vec::new(),
            footer: None,
            image: None,
            thumbnail: None,
            timestamp: None,
        };

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated embed block".into(), line: 0 }),
                Token::Keyword(ref kw) => {
                    match kw.as_str() {
                        "title" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.title = Some(Pattern::Literal(Literal::String(s, false))); self.advance(); }
                        }
                        "description" | "desc" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.description = Some(Pattern::Literal(Literal::String(s, false))); self.advance(); }
                        }
                        "color" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.color = Some(Pattern::Literal(Literal::String(s, false))); self.advance(); }
                            else if let Token::Integer(n) = self.peek() { embed.color = Some(Pattern::Literal(Literal::Integer(n))); self.advance(); }
                        }
                        "field" => {
                            self.advance();
                            let mut name = String::new();
                            let mut value = String::new();
                            let mut inline = false;
                            if let Token::String(s) = self.peek() { name = s; self.advance(); }
                            if let Token::String(s) = self.peek() { value = s; self.advance(); }
                            if let Token::Keyword(ref k) = self.peek() {
                                if k == "inline" {
                                    inline = true;
                                    self.advance();
                                }
                            }
                            embed.fields.push(EmbedField {
                                name,
                                value: Pattern::Literal(Literal::String(value, false)),
                                inline,
                            });
                        }
                        "footer" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.footer = Some(Pattern::Literal(Literal::String(s, false))); self.advance(); }
                        }
                        "image" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.image = Some(s); self.advance(); }
                        }
                        "thumbnail" => {
                            self.advance();
                            if let Token::String(s) = self.peek() { embed.thumbnail = Some(s); self.advance(); }
                        }
                        "timestamp" => {
                            self.advance();
                            embed.timestamp = Some(true);
                        }
                        _ => { self.advance(); }
                    }
                }
                _ => { self.advance(); }
            }
        }

        Ok(embed)
    }

    fn parse_row(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.expect(&Token::LBrace, "expected '{{' for row")?;

        let mut components = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated row block".into(), line: 0 }),
                Token::Keyword(ref kw) => {
                    match kw.as_str() {
                        "button" => {
                            self.advance();
                            let label = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                            let mut style = None;
                            let mut id = None;
                            let mut url = None;
                            let mut disabled = false;
                            let mut emoji = None;

                            loop {
                                match self.peek() {
                                    Token::Keyword(ref k) if k == "style" => {
                                        self.advance();
                                        if let Token::Identifier(s) = self.peek() { style = Some(s); self.advance(); }
                                        else if let Token::String(s) = self.peek() { style = Some(s); self.advance(); }
                                    }
                                    Token::Keyword(ref k) if k == "id" => {
                                        self.advance();
                                        if let Token::Identifier(s) = self.peek() { id = Some(s); self.advance(); }
                                        else if let Token::String(s) = self.peek() { id = Some(s); self.advance(); }
                                    }
                                    Token::Keyword(ref k) if k == "url" => {
                                        self.advance();
                                        if let Token::String(s) = self.peek() { url = Some(s); self.advance(); }
                                    }
                                    Token::Keyword(ref k) if k == "disabled" => {
                                        self.advance();
                                        disabled = true;
                                    }
                                    Token::Keyword(ref k) if k == "emoji" => {
                                        self.advance();
                                        if let Token::String(s) = self.peek() { emoji = Some(s); self.advance(); }
                                    }
                                    _ => break,
                                }
                            }

                            if let Some(url) = url {
                                components.push(Component::UrlButton { label, url, emoji });
                            } else {
                                components.push(Component::Button(Button { label, style, id, url: None, disabled, emoji }));
                            }
                        }
                        "select" => {
                            self.advance();
                            let placeholder = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                            let id = if let Token::Keyword(ref k) = self.peek() { if k == "id" { self.advance(); if let Token::Identifier(s) = self.peek() { let s = s.clone(); self.advance(); s } else if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() } } else { String::new() } } else { String::new() };

                            let mut options = Vec::new();
                            if self.peek() == Token::LBrace {
                                self.advance();
                                loop {
                                    self.consume_newlines();
                                    match self.peek() {
                                        Token::RBrace => { self.advance(); break; }
                                        Token::Keyword(ref k) if k == "option" => {
                                            self.advance();
                                            let label = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                                            let value = if let Token::Keyword(ref k) = self.peek() { if k == "value" { self.advance(); if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else if let Token::Identifier(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() } } else { String::new() } } else { String::new() };
                                            let mut desc = None;
                                            let mut emoji = None;
                                            loop {
                                                match self.peek() {
                                                    Token::Keyword(ref k) if k == "description" || k == "desc" => {
                                                        self.advance();
                                                        if let Token::String(s) = self.peek() { desc = Some(s); self.advance(); }
                                                    }
                                                    Token::Keyword(ref k) if k == "emoji" => {
                                                        self.advance();
                                                        if let Token::String(s) = self.peek() { emoji = Some(s); self.advance(); }
                                                    }
                                                    _ => break,
                                                }
                                            }
                                            options.push(SelectOption { label, value, description: desc, emoji });
                                        }
                                        _ => { self.advance(); }
                                    }
                                }
                            }

                            components.push(Component::Select(Select { placeholder, id, options, min_values: None, max_values: None }));
                        }
                        "url" => {
                            self.advance();
                            let label = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                            let url = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                            components.push(Component::UrlButton { label, url, emoji: None });
                        }
                        _ => { self.advance(); }
                    }
                }
                _ => { self.advance(); }
            }
        }

        Ok(Statement::Row(Row { components }))
    }

    fn parse_modal(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let title = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
        let mut id = None;

        self.expect(&Token::LBrace, "expected '{{' for modal")?;

        let mut inputs = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated modal block".into(), line: 0 }),
                Token::Keyword(ref kw) if kw == "input" => {
                    self.advance();
                    let label = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                    let mut style = "short".into();
                    let mut required = false;
                    let mut placeholder = None;
                    let mut default = None;

                    loop {
                        match self.peek() {
                            Token::Keyword(ref k) if k == "short" || k == "paragraph" => {
                                style = k.clone();
                                self.advance();
                            }
                            Token::Keyword(ref k) if k == "required" => {
                                required = true;
                                self.advance();
                            }
                            Token::Keyword(ref k) if k == "placeholder" => {
                                self.advance();
                                if let Token::String(s) = self.peek() { placeholder = Some(s); self.advance(); }
                            }
                            Token::Keyword(ref k) if k == "default" => {
                                self.advance();
                                if let Token::String(s) = self.peek() { default = Some(s); self.advance(); }
                            }
                            _ => break,
                        }
                    }

                    inputs.push(ModalInput { label, style, required, placeholder, default });
                }
                _ => { self.advance(); }
            }
        }

        Ok(Statement::Modal(Modal { title, id, inputs }))
    }

    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::Keyword(ref k) => { let s = k.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "variable name".into(), found: self.peek(), line: 0 }),
        };

        if self.peek() == Token::Equal {
            self.advance();
            let value = self.parse_pattern()?;
            Ok(Statement::Let { name, value })
        } else {
            Ok(Statement::Let { name, value: Pattern::Literal(Literal::Null) })
        }
    }

    fn parse_set(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let target = self.parse_or()?;
        if self.peek() == Token::Equal {
            self.advance();
            let value = self.parse_pattern()?;
            Ok(Statement::Set { target, value })
        } else {
            Ok(Statement::Set { target, value: Pattern::Literal(Literal::Null) })
        }
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let condition = self.parse_pattern()?;
        self.expect(&Token::LBrace, "expected '{{' for if body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated if block".into(), line: 0 }),
                _ => body.push(self.parse_statement()?),
            }
        }

        let mut elifs = Vec::new();
        let mut else_body = None;

        loop {
            self.consume_newlines();
            match self.peek() {
                Token::Keyword(ref k) if k == "elif" => {
                    self.advance();
                    let cond = self.parse_pattern()?;
                    self.expect(&Token::LBrace, "expected '{{' for elif body")?;
                    let mut el_body = Vec::new();
                    loop {
                        self.consume_newlines();
                        match self.peek() {
                            Token::RBrace => { self.advance(); break; }
                            Token::EOF => return Err(ParseError::Custom { message: "Unterminated elif block".into(), line: 0 }),
                            _ => el_body.push(self.parse_statement()?),
                        }
                    }
                    elifs.push((cond, el_body));
                }
                Token::Keyword(ref k) if k == "else" => {
                    self.advance();
                    self.expect(&Token::LBrace, "expected '{{' for else body")?;
                    let mut el_body = Vec::new();
                    loop {
                        self.consume_newlines();
                        match self.peek() {
                            Token::RBrace => { self.advance(); break; }
                            Token::EOF => return Err(ParseError::Custom { message: "Unterminated else block".into(), line: 0 }),
                            _ => el_body.push(self.parse_statement()?),
                        }
                    }
                    else_body = Some(el_body);
                    break;
                }
                _ => break,
            }
        }

        Ok(Statement::If { condition, body, elifs, else_body })
    }

    fn parse_for(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let var = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            Token::Keyword(ref n) => { let s = n.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "loop variable".into(), found: self.peek(), line: 0 }),
        };

        if let Token::Keyword(ref k) = self.peek() {
            if k == "in" {
                self.advance();
            }
        }

        let iterable = self.parse_pattern()?;
        self.expect(&Token::LBrace, "expected '{{' for for body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated for block".into(), line: 0 }),
                _ => body.push(self.parse_statement()?),
            }
        }

        Ok(Statement::For { var, iterable, body })
    }

    fn parse_thread(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.expect(&Token::LBrace, "expected '{{' for thread body")?;

        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated thread block".into(), line: 0 }),
                _ => body.push(self.parse_statement()?),
            }
        }

        Ok(Statement::Thread { body })
    }

    fn parse_parallel(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let mut branches = Vec::new();

        if self.peek() == Token::LBracket {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBracket => { self.advance(); break; }
                    Token::LBrace => {
                        self.advance();
                        let mut branch = Vec::new();
                        loop {
                            self.consume_newlines();
                            match self.peek() {
                                Token::RBrace => { self.advance(); break; }
                                _ => branch.push(self.parse_statement()?),
                            }
                        }
                        branches.push(branch);
                    }
                    Token::Comma => { self.advance(); }
                    _ => { self.advance(); }
                }
            }
        }

        Ok(Statement::Parallel { branches, result_var: None })
    }

    fn parse_await(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let expr = self.parse_pattern()?;
        Ok(Statement::Await { expr, result_var: None })
    }

    fn parse_try_catch(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        self.expect(&Token::LBrace, "expected '{{' for try body")?;

        let mut try_body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom { message: "Unterminated try block".into(), line: 0 }),
                _ => try_body.push(self.parse_statement()?),
            }
        }

        let mut catch_var = None;
        let mut catch_body = Vec::new();
        let mut finally_body = None;

        self.consume_newlines();
        if let Token::Keyword(ref k) = self.peek() {
            if k == "catch" {
                self.advance();
                if let Token::Identifier(s) = self.peek() {
                    catch_var = Some(s);
                    self.advance();
                }
                self.expect(&Token::LBrace, "expected '{{' for catch body")?;
                loop {
                    self.consume_newlines();
                    match self.peek() {
                        Token::RBrace => { self.advance(); break; }
                        Token::EOF => return Err(ParseError::Custom { message: "Unterminated catch block".into(), line: 0 }),
                        _ => catch_body.push(self.parse_statement()?),
                    }
                }
            }
        }

        self.consume_newlines();
        if let Token::Keyword(ref k) = self.peek() {
            if k == "finally" {
                self.advance();
                self.expect(&Token::LBrace, "expected '{{' for finally body")?;
                let mut fb = Vec::new();
                loop {
                    self.consume_newlines();
                    match self.peek() {
                        Token::RBrace => { self.advance(); break; }
                        Token::EOF => return Err(ParseError::Custom { message: "Unterminated finally block".into(), line: 0 }),
                        _ => fb.push(self.parse_statement()?),
                    }
                }
                finally_body = Some(fb);
            }
        }

        Ok(Statement::TryCatch { try_body, catch_var, catch_body, finally_body })
    }

    fn parse_throw(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let message = self.parse_pattern()?;
        Ok(Statement::Throw { message })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Newline || self.peek() == Token::RBrace || self.peek() == Token::EOF {
            Ok(Statement::Return { value: None })
        } else {
            let value = Some(self.parse_pattern()?);
            Ok(Statement::Return { value })
        }
    }

    fn parse_db(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "db operation".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        match op.as_str() {
            "query" => {
                let query = if let Token::String(s) = self.peek() { let s = s.clone(); self.advance(); s } else { String::new() };
                let mut params = Vec::new();
                loop {
                    match self.peek() {
                        Token::Comma => { self.advance(); }
                        Token::Newline | Token::RBrace | Token::EOF => break,
                        Token::RParen if has_parens => { break; }
                        _ => { params.push(self.parse_pattern()?); }
                    }
                }
                if has_parens { self.expect(&Token::RParen, "expected ')'")?; }
                Ok(Statement::DBQuery { query, params, result_var: None })
            }
            _ => {
                let mut args = Vec::new();
                loop {
                    match self.peek() {
                        Token::Comma => { self.advance(); }
                        Token::Newline | Token::RBrace | Token::EOF => break,
                        Token::RParen if has_parens => { break; }
                        _ => { args.push(self.parse_pattern()?); }
                    }
                }
                if has_parens { self.expect(&Token::RParen, "expected ')'")?; }
                Ok(Statement::DBQuery { query: op, params: args, result_var: None })
            }
        }
    }

    fn parse_http(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let method = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "http method".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let url = self.parse_pattern()?;
        let mut headers = None;
        let mut body = None;

        if self.peek() == Token::LBrace {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBrace => { self.advance(); break; }
                    Token::Keyword(ref k) if k == "headers" => {
                        self.advance();
                        if self.peek() == Token::LBrace {
                            self.advance();
                            let mut h = std::collections::HashMap::new();
                            loop {
                                self.consume_newlines();
                                match self.peek() {
                                    Token::RBrace => { self.advance(); break; }
                                    Token::String(k) => {
                                        let key = k.clone();
                                        self.advance();
                                        if let Token::String(v) = self.peek() {
                                            h.insert(key, Pattern::Literal(Literal::String(v, false)));
                                            self.advance();
                                        }
                                    }
                                    _ => { self.advance(); }
                                }
                            }
                            headers = Some(h);
                        }
                    }
                    Token::Keyword(ref k) if k == "body" => {
                        self.advance();
                        if self.peek() == Token::LBrace {
                            self.advance();
                            let mut b = std::collections::HashMap::new();
                            loop {
                                self.consume_newlines();
                                match self.peek() {
                                    Token::RBrace => { self.advance(); break; }
                                    Token::String(k) => {
                                        let key = k.clone();
                                        self.advance();
                                        let val = self.parse_pattern()?;
                                        b.insert(key, val);
                                    }
                                    _ => { self.advance(); }
                                }
                            }
                            body = Some(b);
                        }
                    }
                    _ => { self.advance(); }
                }
            }
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::HTTPRequest { method, url, headers, body, result_var: None })
    }

    fn parse_json(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "json operation".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let key = self.parse_pattern()?;
        let mut value = None;

        if self.peek() == Token::Comma {
            self.advance();
            value = Some(self.parse_pattern()?);
        } else if !has_parens && (matches!(self.peek(), Token::String(_)) || matches!(self.peek(), Token::Integer(_)) || matches!(self.peek(), Token::Float(_)) || matches!(self.peek(), Token::Boolean(_)) || matches!(self.peek(), Token::Null) || matches!(self.peek(), Token::LBrace) || matches!(self.peek(), Token::LBracket) || matches!(self.peek(), Token::Identifier(_))) {
            value = Some(self.parse_pattern()?);
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::JSONOp { op, key, value, result_var: None })
    }

    fn parse_file(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "file operation".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let path = self.parse_pattern()?;
        let mut content = None;

        if self.peek() == Token::Comma {
            self.advance();
            content = Some(self.parse_pattern()?);
        } else if !has_parens && (matches!(self.peek(), Token::String(_)) || matches!(self.peek(), Token::Identifier(_))) {
            content = Some(self.parse_pattern()?);
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::FileOp { op, path, content, result_var: None })
    }

    fn parse_voice(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "voice operation".into(), found: self.peek(), line: 0 }),
        };

        let mut args = Vec::new();
        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        loop {
            match self.peek() {
                Token::Newline | Token::RBrace | Token::EOF => break,
                Token::Comma => { self.advance(); }
                Token::RParen if has_parens => { break; }
                _ => args.push(self.parse_pattern()?),
            }
        }
        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::VoiceOp { op, args })
    }

    fn parse_webhook(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "webhook operation".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let url = self.parse_pattern()?;
        let mut args = Vec::new();

        if self.peek() == Token::Comma {
            self.advance();
        }

        if self.peek() == Token::LBrace {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBrace => { self.advance(); break; }
                    _ => { args.push(self.parse_pattern()?); }
                }
            }
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::WebhookOp { op, url, args })
    }

    fn parse_cache(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let op = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "cache operation".into(), found: self.peek(), line: 0 }),
        };

        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let key = self.parse_pattern()?;
        let mut value = None;
        let mut ttl = None;

        if self.peek() == Token::Comma {
            self.advance();
            value = Some(self.parse_pattern()?);
        }

        if self.peek() == Token::Comma {
            self.advance();
            if let Token::Integer(n) = self.peek() { ttl = Some(n); self.advance(); }
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::CacheOp { op, key, value, ttl, result_var: None })
    }

    fn parse_paginate(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        let items = self.parse_pattern()?;
        let mut per_page = 10;

        if self.peek() == Token::Comma {
            self.advance();
            if let Token::Integer(n) = self.peek() { per_page = n as i64; self.advance(); }
        }

        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        let mut embed_template = Box::new(Statement::Embed(Embed {
            title: None, description: None, color: None,
            fields: Vec::new(), footer: None, image: None,
            thumbnail: None, timestamp: None,
        }));
        let mut buttons = None;
        let mut timeout = None;

        if self.peek() == Token::LBrace {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBrace => { self.advance(); break; }
                    Token::Keyword(ref k) if k == "embed" => {
                        self.advance();
                        embed_template = Box::new(Statement::Embed(self.parse_embed()?));
                    }
                    Token::Keyword(ref k) if k == "buttons" => {
                        self.advance();
                        if self.peek() == Token::LBracket {
                            self.advance();
                            let mut b = Vec::new();
                            loop {
                                match self.peek() {
                                    Token::RBracket => { self.advance(); break; }
                                    Token::String(s) => { b.push(s); self.advance(); }
                                    Token::Comma => { self.advance(); }
                                    _ => break,
                                }
                            }
                            buttons = Some(b);
                        }
                    }
                    Token::Keyword(ref k) if k == "timeout" => {
                        self.advance();
                        if let Token::Integer(n) = self.peek() { timeout = Some(n); self.advance(); }
                    }
                    _ => { self.advance(); }
                }
            }
        }

        Ok(Statement::Paginate { items, per_page, embed_template, buttons, timeout })
    }

    fn parse_log(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let level = match self.peek() {
            Token::String(s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref k) if k == "info" || k == "warn" || k == "error" || k == "debug" => {
                let s = k.clone();
                self.advance();
                s
            }
            _ => "info".into(),
        };

        let message = self.parse_pattern()?;
        let mut channel = None;

        if let Token::Keyword(ref k) = self.peek() {
            if k == "channel" {
                self.advance();
                channel = Some(self.parse_pattern()?);
            }
        }

        Ok(Statement::Log { level, message, channel })
    }

    fn parse_std(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let module = if self.peek() == Token::Dot { self.advance(); "core".into() } else { "core".into() };
        let func = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "std function name".into(), found: self.peek(), line: 0 }),
        };

        let mut args = Vec::new();
        if self.peek() == Token::LParen {
            self.advance();
            loop {
                match self.peek() {
                    Token::RParen => { self.advance(); break; }
                    Token::Newline | Token::EOF => break,
                    _ => { args.push(self.parse_pattern()?); if self.peek() == Token::Comma { self.advance(); } }
                }
            }
        }

        Ok(Statement::StdCall { module, func, args, result_var: None })
    }

    fn parse_autocomplete(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let options = self.parse_pattern()?;
        Ok(Statement::Autocomplete { options })
    }

    fn parse_mock(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let name = match self.peek() {
            Token::Identifier(n) => { self.advance(); n }
            _ => return Err(ParseError::ExpectedToken { expected: "mock name".into(), found: self.peek(), line: 0 }),
        };

        let mut fields = std::collections::HashMap::new();
        if self.peek() == Token::LBrace {
            self.advance();
            loop {
                self.consume_newlines();
                match self.peek() {
                    Token::RBrace => { self.advance(); break; }
                    Token::Identifier(k) => {
                        let key = k.clone();
                        self.advance();
                        let val = self.parse_pattern()?;
                        fields.insert(key, val);
                    }
                    _ => { self.advance(); }
                }
            }
        }

        Ok(Statement::Mock { name, fields })
    }

    fn parse_assert(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let condition = self.parse_pattern()?;
        let mut message = None;

        if self.peek() == Token::Comma {
            self.advance();
            if let Token::String(s) = self.peek() { message = Some(s); self.advance(); }
        }

        Ok(Statement::Assert { condition, message })
    }

    fn parse_discord(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        if self.peek() == Token::Dot {
            self.advance();
        }
        let action = match self.peek() {
            Token::Identifier(ref s) => { let s = s.clone(); self.advance(); s }
            Token::Keyword(ref s) => { let s = s.clone(); self.advance(); s }
            _ => return Err(ParseError::ExpectedToken { expected: "discord action".into(), found: self.peek(), line: 0 }),
        };

        let mut args = Vec::new();
        let has_parens = self.peek() == Token::LParen;
        if has_parens { self.advance(); }

        loop {
            match self.peek() {
                Token::Newline | Token::RBrace | Token::EOF => break,
                Token::Comma => { self.advance(); }
                Token::RParen if has_parens => { break; }
                _ => args.push(self.parse_pattern()?),
            }
        }
        if has_parens { self.expect(&Token::RParen, "expected ')'")?; }

        Ok(Statement::DiscordAction { action, args })
    }

    fn parse_expression_stmt(&mut self) -> Result<Statement, ParseError> {
        let expr = self.parse_pattern()?;
        Ok(Statement::Expression(expr))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Pattern, ParseError> {
        let left = self.parse_or()?;
        if self.peek() == Token::Equal && self.peek_nth(1) != Token::Equal {
            self.advance();
            let right = self.parse_assignment()?;
            Ok(Pattern::BinOp(Box::new(left), BinOp::Assign, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn parse_or(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_and()?;
        while let Token::Keyword(ref k) = self.peek() {
            if k == "or" {
                self.advance();
                let right = self.parse_and()?;
                left = Pattern::BinOp(Box::new(left), BinOp::Or, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_equality()?;
        while let Token::Keyword(ref k) = self.peek() {
            if k == "and" {
                self.advance();
                let right = self.parse_equality()?;
                left = Pattern::BinOp(Box::new(left), BinOp::And, Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            match self.peek() {
                Token::EqualEqual => { self.advance(); let right = self.parse_comparison()?; left = Pattern::BinOp(Box::new(left), BinOp::Eq, Box::new(right)); }
                Token::BangEqual => { self.advance(); let right = self.parse_comparison()?; left = Pattern::BinOp(Box::new(left), BinOp::Neq, Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Less => { self.advance(); let right = self.parse_term()?; left = Pattern::BinOp(Box::new(left), BinOp::Lt, Box::new(right)); }
                Token::LessEqual => { self.advance(); let right = self.parse_term()?; left = Pattern::BinOp(Box::new(left), BinOp::Lte, Box::new(right)); }
                Token::Greater => { self.advance(); let right = self.parse_term()?; left = Pattern::BinOp(Box::new(left), BinOp::Gt, Box::new(right)); }
                Token::GreaterEqual => { self.advance(); let right = self.parse_term()?; left = Pattern::BinOp(Box::new(left), BinOp::Gte, Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_factor()?;
        loop {
            match self.peek() {
                Token::Plus => { self.advance(); let right = self.parse_factor()?; left = Pattern::BinOp(Box::new(left), BinOp::Add, Box::new(right)); }
                Token::Minus => { self.advance(); let right = self.parse_factor()?; left = Pattern::BinOp(Box::new(left), BinOp::Sub, Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Star => { self.advance(); let right = self.parse_unary()?; left = Pattern::BinOp(Box::new(left), BinOp::Mul, Box::new(right)); }
                Token::Slash => { self.advance(); let right = self.parse_unary()?; left = Pattern::BinOp(Box::new(left), BinOp::Div, Box::new(right)); }
                Token::Percent => { self.advance(); let right = self.parse_unary()?; left = Pattern::BinOp(Box::new(left), BinOp::Mod, Box::new(right)); }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Token::Bang => { self.advance(); let right = self.parse_unary()?; Ok(Pattern::BinOp(Box::new(right), BinOp::Eq, Box::new(Pattern::Literal(Literal::Boolean(false))))) }
            Token::Minus => { self.advance(); let right = self.parse_unary()?; Ok(Pattern::BinOp(Box::new(Pattern::Literal(Literal::Integer(0))), BinOp::Sub, Box::new(right))) }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Pattern, ParseError> {
        let mut left = self.parse_primary()?;

        loop {
            match self.peek() {
                Token::LParen => {
                    self.advance();
                    let mut args = Vec::new();
                    loop {
                        match self.peek() {
                            Token::RParen => { self.advance(); break; }
                            Token::Newline | Token::EOF => break,
                            _ => { args.push(self.parse_pattern()?); if self.peek() == Token::Comma { self.advance(); } }
                        }
                    }
                    left = Pattern::Call(Box::new(left), args);
                }
                Token::Dot => {
                    self.advance();
                    if let Token::Identifier(name) = self.peek() {
                        let n = name.clone();
                        self.advance();
                        left = Pattern::MemberAccess(Box::new(left), n);
                    } else if let Token::Keyword(ref name) = self.peek() {
                        let n = name.clone();
                        self.advance();
                        left = Pattern::MemberAccess(Box::new(left), n);
                    } else {
                        break;
                    }
                }
                Token::LBracket => {
                    self.advance();
                    let index = self.parse_pattern()?;
                    self.expect(&Token::RBracket, "expected ']'")?;
                    left = Pattern::Index(Box::new(left), Box::new(index));
                }
                _ => break,
            }
        }

        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Pattern, ParseError> {
        match self.peek() {
            Token::String(s) => {
                self.advance();
                Ok(Pattern::Literal(Literal::String(s, false)))
            }
            Token::InterpolatedString(_s, parts) => {
                self.advance();
                let mut exprs = Vec::new();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 0 {
                        exprs.push(Pattern::Literal(Literal::String(part.clone(), false)));
                    } else {
                        let tokens = crate::lexer::Lexer::new(part).tokenize().map_err(|e| {
                            ParseError::Custom { message: format!("Interpolation error: {}", e), line: 0 }
                        })?;
                        let expr = Parser::new(tokens).parse_pattern()?;
                        exprs.push(expr);
                    }
                }
                Ok(Pattern::Interpolated(exprs))
            }
            Token::Integer(n) => { self.advance(); Ok(Pattern::Literal(Literal::Integer(n))) }
            Token::Float(f) => { self.advance(); Ok(Pattern::Literal(Literal::Float(f))) }
            Token::Boolean(b) => { self.advance(); Ok(Pattern::Literal(Literal::Boolean(b))) }
            Token::Null => { self.advance(); Ok(Pattern::Literal(Literal::Null)) }
            Token::Identifier(name) => { self.advance(); Ok(Pattern::Variable(name)) }
            Token::Keyword(ref k) if k == "env" => {
                self.advance();
                let var_name = if self.peek() == Token::LParen {
                    self.advance();
                    let name = match self.peek() {
                        Token::String(s) => { let s = s.clone(); self.advance(); s }
                        _ => String::new(),
                    };
                    self.expect(&Token::RParen, "expected ')'")?;
                    name
                } else if let Token::String(s) = self.peek() {
                    let s = s.clone();
                    self.advance();
                    s
                } else {
                    String::new()
                };
                Ok(Pattern::Call(
                    Box::new(Pattern::Variable("env".into())),
                    vec![Pattern::Literal(Literal::String(var_name, false))]
                ))
            }
            Token::Keyword(ref k) => {
                let kw = k.clone();
                self.advance();
                Ok(Pattern::Variable(kw))
            }
            Token::LBrace => {
                self.advance();
                let mut fields = std::collections::HashMap::new();
                loop {
                    self.consume_newlines();
                    match self.peek() {
                        Token::RBrace => { self.advance(); break; }
                        Token::Identifier(k) => {
                            let key = k.clone();
                            self.advance();
                            let val = if self.peek() == Token::Colon {
                                self.advance();
                                self.parse_pattern()?
                            } else {
                                Pattern::Variable(key.clone())
                            };
                            fields.insert(key, val);
                            if self.peek() == Token::Comma { self.advance(); }
                        }
                        Token::String(k) => {
                            let key = k.clone();
                            self.advance();
                            let val = if self.peek() == Token::Colon {
                                self.advance();
                                self.parse_pattern()?
                            } else {
                                Pattern::Literal(Literal::String(key.clone(), false))
                            };
                            fields.insert(key, val);
                            if self.peek() == Token::Comma { self.advance(); }
                        }
                        _ => { self.advance(); }
                    }
                }
                Ok(Pattern::Object(fields))
            }
            Token::LBracket => {
                self.advance();
                let mut items = Vec::new();
                loop {
                    match self.peek() {
                        Token::RBracket => { self.advance(); break; }
                        Token::Newline => { self.consume_newlines(); }
                        _ => { items.push(self.parse_pattern()?); if self.peek() == Token::Comma { self.advance(); } }
                    }
                }
                Ok(Pattern::List(items))
            }
            Token::LParen => {
                self.advance();
                let expr = self.parse_pattern()?;
                self.expect(&Token::RParen, "expected ')'")?;
                Ok(expr)
            }
            _ => Err(ParseError::ExpectedToken {
                expected: "expression".into(),
                found: self.peek(),
                line: 0,
            }),
        }
    }

    // ---- Parsing for new feature statements (while, break, continue, match) ----
    // These are handled inside parse_statement() via the keyword match above.

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let condition = self.parse_pattern()?;
        self.expect(&Token::LBrace, "expected '{' for while body")?;
        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom {
                    message: "Unterminated while block".into(), line: 0,
                }),
                _ => body.push(self.parse_statement()?),
            }
        }
        Ok(Statement::While { condition, body })
    }

    fn parse_match(&mut self) -> Result<Statement, ParseError> {
        self.advance();
        let value = self.parse_pattern()?;
        self.expect(&Token::LBrace, "expected '{' for match")?;
        let mut arms = Vec::new();
        let mut else_body = None;
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => break,
                Token::Keyword(ref k2) if k2 == "else" => {
                    self.advance();
                    self.expect(&Token::LBrace, "expected '{' for else branch")?;
                    let mut eb = Vec::new();
                    loop {
                        self.consume_newlines();
                        match self.peek() {
                            Token::RBrace => { self.advance(); break; }
                            _ => eb.push(self.parse_statement()?),
                        }
                    }
                    else_body = Some(eb);
                }
                _ => {
                    let pat = self.parse_pattern()?;
                    self.expect(&Token::LBrace, "expected '{' for match arm")?;
                    let mut arm_body = Vec::new();
                    loop {
                        self.consume_newlines();
                        match self.peek() {
                            Token::RBrace => { self.advance(); break; }
                            _ => arm_body.push(self.parse_statement()?),
                        }
                    }
                    arms.push((pat, arm_body));
                }
            }
        }
        Ok(Statement::Match { value, arms, else_body })
    }

    fn parse_type_name(&mut self) -> Result<TypeName, ParseError> {
        match self.peek() {
            Token::Identifier(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(match s.as_str() {
                    "string" => TypeName::String,
                    "int" => TypeName::Int,
                    "float" => TypeName::Float,
                    "bool" => TypeName::Bool,
                    "user" => TypeName::User,
                    "channel" => TypeName::Channel,
                    "role" => TypeName::Role,
                    "member" => TypeName::Member,
                    "attachment" => TypeName::Attachment,
                    _ => TypeName::Custom(s),
                })
            }
            Token::Keyword(ref s) if s == "string" || s == "int" || s == "float" || s == "bool"
                || s == "user" || s == "channel" || s == "role" || s == "member" || s == "attachment" =>
            {
                let s = s.clone();
                self.advance();
                Ok(match s.as_str() {
                    "string" => TypeName::String,
                    "int" => TypeName::Int,
                    "float" => TypeName::Float,
                    "bool" => TypeName::Bool,
                    "user" => TypeName::User,
                    "channel" => TypeName::Channel,
                    "role" => TypeName::Role,
                    "member" => TypeName::Member,
                    "attachment" => TypeName::Attachment,
                    _ => unreachable!(),
                })
            }
            _ => Err(ParseError::ExpectedToken {
                expected: "type name (string, int, float, bool, user, ...)".into(),
                found: self.peek(),
                line: 0,
            }),
        }
    }

    fn parse_function_def(&mut self) -> Result<Function, ParseError> {
        let name = match self.peek() {
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken {
                expected: "function name".into(), found: self.peek(), line: 0,
            }),
        };

        let mut params = Vec::new();
        if self.peek() == Token::LParen {
            self.advance();
            self.consume_newlines();
            while self.peek() != Token::RParen && self.peek() != Token::EOF {
                let pname = match self.peek() {
                    Token::Identifier(s) => { self.advance(); s }
                    _ => break,
                };
                let mut param_type = TypeName::String;
                if let Ok(t) = self.parse_type_name() {
                    param_type = t;
                }
                let mut default = None;
                if let Token::Keyword(ref k) = self.peek() {
                    if k == "default" {
                        self.advance();
                        default = Some(self.parse_pattern()?);
                    }
                }
                params.push(FnParam { name: pname, param_type, default });
                if self.peek() == Token::Comma {
                    self.advance();
                    self.consume_newlines();
                }
            }
            if self.peek() == Token::RParen { self.advance(); }
        }

        let mut return_type = None;
        if let Token::Arrow = self.peek() {
            self.advance();
            return_type = Some(self.parse_type_name()?);
        }

        self.expect(&Token::LBrace, "expected '{' for function body")?;
        let mut body = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom {
                    message: "Unterminated function block".into(), line: 0,
                }),
                _ => body.push(self.parse_statement()?),
            }
        }

        Ok(Function { name, params, return_type, body })
    }

    fn parse_enum_def(&mut self) -> Result<EnumDef, ParseError> {
        let name = match self.peek() {
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken {
                expected: "enum name".into(), found: self.peek(), line: 0,
            }),
        };

        self.expect(&Token::LBrace, "expected '{' for enum definition")?;
        let mut variants = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom {
                    message: "Unterminated enum block".into(), line: 0,
                }),
                _ => {
                    let vname = match self.peek() {
                        Token::Identifier(s) => { self.advance(); s }
                        _ => break,
                    };
                    let mut fields = Vec::new();
                    if self.peek() == Token::LParen {
                        self.advance();
                        loop {
                            self.consume_newlines();
                            match self.peek() {
                                Token::RParen => { self.advance(); break; }
                                Token::EOF => break,
                                _ => {
                                    let tok = self.peek().clone();
                                    self.advance();
                                    let (id, kw) = match tok {
                                        Token::Identifier(s) => (Some(s), None),
                                        Token::Keyword(s) => (None, Some(s)),
                                        _ => break,
                                    };
                                    let field_name;
                                    let field_type;
                                    if self.peek() == Token::Comma || self.peek() == Token::RParen {
                                        // Single token = type name
                                        field_name = String::new();
                                        let s = id.or(kw).unwrap_or_default();
                                        field_type = match s.as_str() {
                                            "string" => TypeName::String,
                                            "int" => TypeName::Int,
                                            "float" => TypeName::Float,
                                            "bool" => TypeName::Bool,
                                            "user" => TypeName::User,
                                            "channel" => TypeName::Channel,
                                            "role" => TypeName::Role,
                                            "member" => TypeName::Member,
                                            "attachment" => TypeName::Attachment,
                                            s => TypeName::Custom(s.into()),
                                        };
                                    } else {
                                        // Two tokens = name: type
                                        field_name = id.or(kw).unwrap_or_default();
                                        field_type = self.parse_type_name()?;
                                    }
                                    fields.push(EnumField { name: field_name, field_type });
                                    if self.peek() == Token::Comma {
                                        self.advance();
                                    }
                                }
                            }
                        }
                    }
                    variants.push(EnumVariant { name: vname, fields });
                }
            }
        }

        Ok(EnumDef { name, variants })
    }

    fn parse_migration(&mut self) -> Result<Migration, ParseError> {
        let table = match self.peek() {
            Token::Identifier(s) => { self.advance(); s }
            Token::String(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken {
                expected: "table name".into(), found: self.peek(), line: 0,
            }),
        };

        self.expect(&Token::LBrace, "expected '{' for table definition")?;
        let mut columns = Vec::new();
        let mut indexes = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => return Err(ParseError::Custom {
                    message: "Unterminated table block".into(), line: 0,
                }),
                Token::Keyword(ref k) if k == "index" => {
                    self.advance();
                    let iname = match self.peek() {
                        Token::Identifier(s) => { self.advance(); s }
                        _ => return Err(ParseError::ExpectedToken {
                            expected: "index name".into(), found: self.peek(), line: 0,
                        }),
                    };
                    self.expect(&Token::LParen, "expected '(' for index columns")?;
                    let mut icols = Vec::new();
                    loop {
                        match self.peek() {
                            Token::RParen => { self.advance(); break; }
                            Token::Identifier(s) => { icols.push(s); self.advance(); }
                            Token::Comma => { self.advance(); }
                            _ => break,
                        }
                    }
                    let mut unique = false;
                    if let Token::Keyword(ref k) = self.peek() {
                        if k == "unique" { unique = true; self.advance(); }
                    }
                    indexes.push(IndexDef { name: iname, columns: icols, unique });
                }
                _ => {
                    let cname = match self.peek() {
                        Token::Identifier(s) => { self.advance(); s }
                        _ => { self.advance(); continue; }
                    };
                    let col_type = match self.peek() {
                        Token::Identifier(s) => { let s = s.clone(); self.advance(); s }
                        _ => "string".into(),
                    };
                    let mut primary_key = false;
                    let mut auto_increment = false;
                    let mut not_null = false;
                    let mut unique = false;
                    let mut default = None;
                    let mut references = None;
                    loop {
                        match self.peek() {
                            Token::Keyword(ref k) if k == "primary" => {
                                self.advance();
                                if let Token::Keyword(ref k2) = self.peek() {
                                    if k2 == "key" { self.advance(); }
                                }
                                primary_key = true;
                            }
                            Token::Keyword(ref k) if k == "auto_increment" => {
                                auto_increment = true; self.advance();
                            }
                            Token::Keyword(ref k) if k == "not_null" || k == "not" => {
                                not_null = true; self.advance();
                                if k == "not" {
                                    if let Token::Keyword(ref k2) = self.peek() {
                                        if k2 == "null" { self.advance(); }
                                    }
                                }
                            }
                            Token::Keyword(ref k) if k == "unique" => {
                                unique = true; self.advance();
                            }
                            Token::Keyword(ref k) if k == "default" => {
                                self.advance();
                                default = Some(self.parse_pattern()?);
                            }
                            Token::Keyword(ref k) if k == "references" => {
                                self.advance();
                                let ref_table = match self.peek() {
                                    Token::Identifier(s) => { self.advance(); s }
                                    _ => break,
                                };
                                let ref_col = match self.peek() {
                                    Token::LParen => {
                                        self.advance();
                                        let col = match self.peek() {
                                            Token::Identifier(s) => { self.advance(); s }
                                            _ => "id".into(),
                                        };
                                        if self.peek() == Token::RParen { self.advance(); }
                                        col
                                    }
                                    _ => "id".into(),
                                };
                                references = Some((ref_table, ref_col));
                            }
                            _ => break,
                        }
                    }
                    columns.push(ColumnDef {
                        name: cname, col_type, primary_key, auto_increment,
                        not_null, unique, default, references,
                    });
                }
            }
        }
        Ok(Migration { table, columns, indexes })
    }

    fn parse_model(&mut self) -> Result<Model, ParseError> {
        let name = match self.peek() {
            Token::Identifier(s) => { self.advance(); s }
            _ => return Err(ParseError::ExpectedToken {
                expected: "model name".into(), found: self.peek(), line: 0,
            }),
        };
        let table = match self.peek() {
            Token::String(s) => { self.advance(); s }
            _ => name.clone(),
        };
        self.expect(&Token::LBrace, "expected '{' for model definition")?;
        let mut fields = Vec::new();
        loop {
            self.consume_newlines();
            match self.peek() {
                Token::RBrace => { self.advance(); break; }
                Token::EOF => break,
                _ => {
                    let fname = match self.peek() {
                        Token::Identifier(s) => { self.advance(); s }
                        _ => break,
                    };
                    let field_type = self.parse_type_name()?;
                    let mut default = None;
                    let mut optional = false;
                    if let Token::Keyword(ref k) = self.peek() {
                        if k == "default" {
                            self.advance();
                            match self.peek() {
                                Token::String(s) => { default = Some(Literal::String(s, false)); self.advance(); }
                                Token::Integer(n) => { default = Some(Literal::Integer(n)); self.advance(); }
                                Token::Float(f) => { default = Some(Literal::Float(f)); self.advance(); }
                                Token::Boolean(b) => { default = Some(Literal::Boolean(b)); self.advance(); }
                                _ => {}
                            }
                        }
                    }
                    if let Token::Keyword(ref k) = self.peek() {
                        if k == "optional" { optional = true; self.advance(); }
                    }
                    fields.push(TypeField { name: fname, field_type, default, optional });
                }
            }
        }
        Ok(Model { name, table, fields })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse(input: &str) -> Program {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        parser.parse().unwrap()
    }

    #[test]
    fn test_simple_command() {
        let program = parse(r#"
cmd ping {
    slash true
    reply "Pong!"
}
"#);
        assert_eq!(program.commands.len(), 1);
        assert_eq!(program.commands[0].name, "ping");
        assert!(program.commands[0].slash);
    }

    #[test]
    fn test_config() {
        let program = parse(r#"
config {
    prefix "!"
    threads 4
}
"#);
        assert!(program.config.is_some());
        assert_eq!(program.config.as_ref().unwrap().prefix, Some("!".into()));
    }

    #[test]
    fn test_command_with_params() {
        let program = parse(r#"
cmd greet {
    slash true
    param name {
        type string
        desc "Nome do usuário"
    }
    reply "Olá {name}!"
}
"#);
        assert_eq!(program.commands[0].params.len(), 1);
        assert_eq!(program.commands[0].params[0].name, "name");
    }

    #[test]
    fn test_embed_command() {
        let program = parse(r##"
cmd info {
    slash true
    embed {
        title "Info"
        desc "Bot info"
        color "#FF0000"
        field "Versao" "1.0" inline true
        footer "footer"
        timestamp true
    }
}
"##);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_if_else_command() {
        let program = parse(r#"
cmd test {
    let x = 10
    if x > 5 {
        reply "maior"
    } else {
        reply "menor"
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_for_loop() {
        let program = parse(r#"
cmd list {
    for item in items {
        reply item
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_try_catch() {
        let program = parse(r#"
cmd safe {
    try {
        throw "erro"
    } catch e {
        reply "erro: {e}"
    } finally {
        log info "fim"
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_middleware_def() {
        let program = parse(r#"
middleware "logging" {
    log info "executou"
}
"#);
        assert_eq!(program.middleware.len(), 1);
        assert_eq!(program.middleware[0].name, "logging");
    }

    #[test]
    fn test_event() {
        let program = parse(r#"
on member_join {
    reply "bem-vindo!"
}
"#);
        assert_eq!(program.events.len(), 1);
        assert_eq!(program.events[0].name, "member_join");
    }

    #[test]
    fn test_schedule() {
        let program = parse(r#"
schedule "cleanup" {
    cron "0 0 * * *"
    log info "limpeza"
}
"#);
        assert_eq!(program.schedules.len(), 1);
    }

    #[test]
    fn test_import() {
        let program = parse(r#"import "mod" as util"#);
        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.imports[0].alias, Some("util".into()));
    }

    #[test]
    fn test_context_menu() {
        let program = parse(r#"
menu user "User Info" {
    reply "info"
}
"#);
        assert_eq!(program.context_menus.len(), 1);
    }

    #[test]
    fn test_custom_type() {
        let program = parse(r#"
type Config {
    prefix string default "!"
    log_channel string optional
}
"#);
        assert_eq!(program.types.len(), 1);
    }

    #[test]
    fn test_subcommand() {
        let program = parse(r#"
cmd config {
    sub set {
        reply "ok"
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
        assert_eq!(program.commands[0].subcommands.len(), 1);
    }

    #[test]
    fn test_binary_ops() {
        let program = parse(r#"
cmd math {
    let x = 1 + 2 * 3
    let y = 10 > 5 and true
    let z = x == y or false
    reply "{x} {y} {z}"
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_list_and_object() {
        let program = parse(r#"
cmd data {
    let list = [1, 2, 3]
    let obj = { name: "foo", value: 42 }
    reply list
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_paginate() {
        let program = parse(r#"
cmd list {
    paginate(items, 5) {
        embed { title "Página {page}" }
        buttons ["<", ">"]
        timeout 30
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_row_with_buttons() {
        let program = parse(r#"
cmd menu {
    row {
        button "Sim" style success id "yes"
        url "Site" "https://example.com"
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_modal() {
        let program = parse(r#"
cmd feedback {
    modal "Feedback" {
        input "Nome" short required
        input "Mensagem" paragraph
    }
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_db_query() {
        let program = parse(r#"
cmd db_test {
    let rows = db.query("SELECT * FROM users")
    reply rows
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_discord_action() {
        let program = parse(r#"
cmd kick_user {
    discord.kick(user, "reason")
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_json_op() {
        let program = parse(r#"
cmd json_test {
    json.set("key", "value")
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_http_request() {
        let program = parse(r#"
cmd http_test {
    http.get "https://api.example.com"
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_cooldown_and_permission() {
        let program = parse(r#"
cmd admin {
    slash true
    permission administrator
    cooldown 10 "s" per user
    reply "admin only"
}
"#);
        assert_eq!(program.commands.len(), 1);
        assert_eq!(program.commands[0].permission, Some("administrator".into()));
    }

    #[test]
    fn test_test_block() {
        let program = parse(r#"
test "math" {
    assert 1 + 1 == 2, "should be 2"
    assert std.len("hi") == 2
}
"#);
        assert_eq!(program.tests.len(), 1);
    }

    #[test]
    fn test_thread_and_parallel() {
        let program = parse(r#"
cmd concurrent {
    thread {
        reply "background"
    }
    parallel [{
        reply "a"
    }, {
        reply "b"
    }]
}
"#);
        assert_eq!(program.commands.len(), 1);
    }

    #[test]
    fn test_let_keyword_variable() {
        let program = parse(r#"
cmd vars {
    let info = "dados"
    reply info
}
"#);
        assert_eq!(program.commands[0].body.len(), 2);
    }

    #[test]
    fn test_std_call() {
        let program = parse(r#"
cmd test {
    let r = std.random(1, 10)
    let u = std.upper("hello")
    reply r
}
"#);
        assert_eq!(program.commands.len(), 1);
    }
}
