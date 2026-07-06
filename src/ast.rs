use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String, bool),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeName {
    String,
    Int,
    Float,
    Bool,
    User,
    Channel,
    Role,
    Member,
    Attachment,
    Custom(String),
    Enum(String),
    Table(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, Neq, Lt, Gt, Lte, Gte,
    And, Or,
    Assign,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Variable(String),
    MemberAccess(Box<Pattern>, String),
    Index(Box<Pattern>, Box<Pattern>),
    Call(Box<Pattern>, Vec<Pattern>),
    BinOp(Box<Pattern>, BinOp, Box<Pattern>),
    List(Vec<Pattern>),
    Object(HashMap<String, Pattern>),
    Interpolated(Vec<Pattern>),
    EnumVariant(String, String, Option<HashMap<String, Pattern>>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub param_type: TypeName,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub choices: Option<Vec<String>>,
    pub autocomplete: bool,
    pub default: Option<Literal>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnParam {
    pub name: String,
    pub param_type: TypeName,
    pub default: Option<Pattern>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmbedField {
    pub name: String,
    pub value: Pattern,
    pub inline: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Embed {
    pub title: Option<Pattern>,
    pub description: Option<Pattern>,
    pub color: Option<Pattern>,
    pub fields: Vec<EmbedField>,
    pub footer: Option<Pattern>,
    pub image: Option<String>,
    pub thumbnail: Option<String>,
    pub timestamp: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Button {
    pub label: String,
    pub style: Option<String>,
    pub id: Option<String>,
    pub url: Option<String>,
    pub disabled: bool,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
    pub description: Option<String>,
    pub emoji: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub placeholder: String,
    pub id: String,
    pub options: Vec<SelectOption>,
    pub min_values: Option<i64>,
    pub max_values: Option<i64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ModalInput {
    pub label: String,
    pub style: String,
    pub required: bool,
    pub placeholder: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Modal {
    pub title: String,
    pub id: Option<String>,
    pub inputs: Vec<ModalInput>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Component {
    Button(Button),
    Select(Select),
    UrlButton { label: String, url: String, emoji: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    FnCall {
        name: String,
        args: Vec<Pattern>,
        result_var: Option<String>,
    },
    Reply {
        content: Pattern,
        ephemeral: Option<bool>,
        components: Option<bool>,
    },
    Send {
        content: Pattern,
        channel: Option<Pattern>,
    },
    Embed(Embed),
    Row(Row),
    Modal(Modal),
    Let {
        name: String,
        value: Pattern,
    },
    Set {
        target: Pattern,
        value: Pattern,
    },
    If {
        condition: Pattern,
        body: Vec<Statement>,
        elifs: Vec<(Pattern, Vec<Statement>)>,
        else_body: Option<Vec<Statement>>,
    },
    For {
        var: String,
        iterable: Pattern,
        body: Vec<Statement>,
    },
    Thread {
        body: Vec<Statement>,
    },
    Parallel {
        branches: Vec<Vec<Statement>>,
        result_var: Option<String>,
    },
    Await {
        expr: Pattern,
        result_var: Option<String>,
    },
    TryCatch {
        try_body: Vec<Statement>,
        catch_var: Option<String>,
        catch_body: Vec<Statement>,
        finally_body: Option<Vec<Statement>>,
    },
    Throw {
        message: Pattern,
    },
    Return {
        value: Option<Pattern>,
    },
    Cancel,
    Expression(Pattern),
    CustomEvent {
        name: String,
        args: HashMap<String, Pattern>,
    },
    DiscordAction {
        action: String,
        args: Vec<Pattern>,
    },
    DBQuery {
        query: String,
        params: Vec<Pattern>,
        result_var: Option<String>,
    },
    HTTPRequest {
        method: String,
        url: Pattern,
        headers: Option<HashMap<String, Pattern>>,
        body: Option<HashMap<String, Pattern>>,
        result_var: Option<String>,
    },
    JSONOp {
        op: String,
        key: Pattern,
        value: Option<Pattern>,
        result_var: Option<String>,
    },
    FileOp {
        op: String,
        path: Pattern,
        content: Option<Pattern>,
        result_var: Option<String>,
    },
    VoiceOp {
        op: String,
        args: Vec<Pattern>,
    },
    WebhookOp {
        op: String,
        url: Pattern,
        args: Vec<Pattern>,
    },
    CacheOp {
        op: String,
        key: Pattern,
        value: Option<Pattern>,
        ttl: Option<i64>,
        result_var: Option<String>,
    },
    StdCall {
        module: String,
        func: String,
        args: Vec<Pattern>,
        result_var: Option<String>,
    },
    Paginate {
        items: Pattern,
        per_page: i64,
        embed_template: Box<Statement>,
        buttons: Option<Vec<String>>,
        timeout: Option<i64>,
    },
    Log {
        level: String,
        message: Pattern,
        channel: Option<Pattern>,
    },
    Autocomplete {
        options: Pattern,
    },
    CancelEvent,
    Mock {
        name: String,
        fields: HashMap<String, Pattern>,
    },
    Assert {
        condition: Pattern,
        message: Option<String>,
    },
    Execute {
        command: String,
        result_var: Option<String>,
    },
    Break,
    Continue,
    While {
        condition: Pattern,
        body: Vec<Statement>,
    },
    Match {
        value: Pattern,
        arms: Vec<(Pattern, Vec<Statement>)>,
        else_body: Option<Vec<Statement>>,
    },
    TryExpr {
        body: Box<Statement>,
        catch_var: Option<String>,
        catch_body: Box<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Command {
    pub name: String,
    pub slash: bool,
    pub prefix: Option<String>,
    pub description: Option<String>,
    pub guild_only: bool,
    pub permission: Option<String>,
    pub cooldown: Option<Cooldown>,
    pub middleware: Vec<String>,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
    pub subcommands: Vec<Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cooldown {
    pub duration: i64,
    pub unit: String,
    pub scope: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schedule {
    pub name: String,
    pub cron: Option<String>,
    pub every: Option<i64>,
    pub every_unit: Option<String>,
    pub once_after: Option<i64>,
    pub once_unit: Option<String>,
    pub at_time: Option<String>,
    pub timezone: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub name: String,
    pub filter: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MiddlewareDef {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub kind: String,
    pub name: String,
    pub permission: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeField {
    pub name: String,
    pub field_type: TypeName,
    pub default: Option<Literal>,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomType {
    pub name: String,
    pub fields: Vec<TypeField>,
    pub methods: Vec<Function>,
    pub table_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub params: Vec<FnParam>,
    pub return_type: Option<TypeName>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub fields: Vec<EnumField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumField {
    pub name: String,
    pub field_type: TypeName,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: String,
    pub primary_key: bool,
    pub auto_increment: bool,
    pub not_null: bool,
    pub unique: bool,
    pub default: Option<Pattern>,
    pub references: Option<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexDef {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Migration {
    pub table: String,
    pub columns: Vec<ColumnDef>,
    pub indexes: Vec<IndexDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub name: String,
    pub table: String,
    pub fields: Vec<TypeField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub prefix: Option<String>,
    pub status: Option<(String, String)>,
    pub intents: Vec<String>,
    pub threads: Option<i64>,
    pub http_timeout: Option<i64>,
    pub http_retry: Option<i64>,
    pub db_url: Option<String>,
    pub db_auto_migrate: Option<bool>,
    pub cache_ttl: Option<i64>,
    pub cache_max_size: Option<String>,
    pub log_level: Option<String>,
    pub log_file: Option<String>,
    pub log_format: Option<String>,
    pub error_channel: Option<String>,
    pub error_ephemeral: Option<bool>,
    pub allowed_paths: Vec<String>,
    pub max_file_size: Option<String>,
    pub global_middleware: Vec<String>,
    pub token_env: Option<String>,
    pub env_prefix: Option<String>,
    pub db_models: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub config: Option<Config>,
    pub imports: Vec<Import>,
    pub commands: Vec<Command>,
    pub events: Vec<Event>,
    pub schedules: Vec<Schedule>,
    pub middleware: Vec<MiddlewareDef>,
    pub context_menus: Vec<ContextMenu>,
    pub types: Vec<CustomType>,
    pub tests: Vec<TestCase>,
    pub functions: Vec<Function>,
    pub enums: Vec<EnumDef>,
    pub migrations: Vec<Migration>,
    pub models: Vec<Model>,
}
