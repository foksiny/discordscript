use crate::ast::*;
use std::collections::HashSet;

const VALID_INTENTS: &[&str] = &[
    "guilds", "members", "bans", "emojis", "integrations", "webhooks",
    "invites", "voice_states", "presences", "messages", "reactions",
    "typing", "moderation", "message_content", "guild_scheduled_events",
];

const VALID_LOG_LEVELS: &[&str] = &["debug", "info", "warn", "error"];

const VALID_METHODS: &[&str] = &["get", "post", "put", "patch", "delete", "head", "options"];

const VALID_JSON_OPS: &[&str] = &["set", "get", "delete", "push"];

const VALID_CACHE_OPS: &[&str] = &["set", "get", "delete", "clear", "stats"];

const VALID_DISCORD_ACTIONS: &[&str] = &[
    "ban", "kick", "mute", "unmute", "deafen", "undeafen",
    "move", "timeout", "add_role", "remove_role",
];

const VALID_CONTEXT_MENU_KINDS: &[&str] = &["user", "message"];

pub struct Checker<'a> {
    program: &'a Program,
}

impl<'a> Checker<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            program,
        }
    }

    pub fn check(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        self.check_duplicates(&mut warnings);
        self.check_triggers(&mut warnings);
        self.check_params(&mut warnings);
        self.check_config(&mut warnings);
        self.check_schedules(&mut warnings);
        self.check_middleware_refs(&mut warnings);
        self.check_cooldowns(&mut warnings);
        self.check_context_menus(&mut warnings);

        warnings
    }

    fn check_duplicates(&self, warnings: &mut Vec<String>) {
        let mut cmd_names = HashSet::new();
        for cmd in &self.program.commands {
            if !cmd_names.insert(&cmd.name) {
                warnings.push(format!("Duplicate command name: '{}'", cmd.name));
            }
        }

        let mut evt_names = HashSet::new();
        for evt in &self.program.events {
            if !evt_names.insert(&evt.name) {
                warnings.push(format!("Duplicate event name: '{}'", evt.name));
            }
        }

        let mut sched_names = HashSet::new();
        for sched in &self.program.schedules {
            if !sched_names.insert(&sched.name) {
                warnings.push(format!("Duplicate schedule name: '{}'", sched.name));
            }
        }

        let mut type_names = HashSet::new();
        for ty in &self.program.types {
            if !type_names.insert(&ty.name) {
                warnings.push(format!("Duplicate type name: '{}'", ty.name));
            }
        }

        let mut mw_names = HashSet::new();
        for mw in &self.program.middleware {
            if !mw_names.insert(&mw.name) {
                warnings.push(format!("Duplicate middleware name: '{}'", mw.name));
            }
        }
    }

    fn check_triggers(&self, warnings: &mut Vec<String>) {
        for cmd in &self.program.commands {
            if !cmd.slash && cmd.prefix.is_none() {
                warnings.push(format!("Command '{}' has no slash or prefix trigger", cmd.name));
            }
        }
    }

    fn check_params(&self, warnings: &mut Vec<String>) {
        for cmd in &self.program.commands {
            self.check_command_params(cmd, warnings);
            for sub in &cmd.subcommands {
                self.check_command_params(sub, warnings);
            }
        }
    }

    fn check_command_params(&self, cmd: &Command, warnings: &mut Vec<String>) {
        for param in &cmd.params {
            if param.required.unwrap_or(false) && param.default.is_some() {
                warnings.push(format!(
                    "Command '{}' param '{}' is required but has a default value",
                    cmd.name, param.name
                ));
            }
        }
    }

    fn check_config(&self, warnings: &mut Vec<String>) {
        if let Some(ref config) = self.program.config {
            // Validate intents
            for intent in &config.intents {
                if !VALID_INTENTS.contains(&intent.as_str()) {
                    warnings.push(format!("Unknown intent: '{}'", intent));
                }
            }

            // Validate log level
            if let Some(ref level) = config.log_level {
                if !VALID_LOG_LEVELS.contains(&level.as_str()) {
                    warnings.push(format!(
                        "Invalid log level: '{}'. Valid: {:?}",
                        level, VALID_LOG_LEVELS
                    ));
                }
            }

            // Validate error_channel should start with #
            if let Some(ref channel) = config.error_channel {
                if !channel.starts_with('#') {
                    warnings.push(format!("error_channel should start with '#' (got '{}')", channel));
                }
            }

            // Validate allowed_paths don't contain '..'
            for path in &config.allowed_paths {
                if path.contains("..") {
                    warnings.push(format!("allowed_path '{}' contains '..' (security risk)", path));
                }
            }
        }
    }

    fn check_schedules(&self, warnings: &mut Vec<String>) {
        for sched in &self.program.schedules {
            if let Some(ref cron) = sched.cron {
                let parts: Vec<&str> = cron.split_whitespace().collect();
                if parts.len() != 5 {
                    warnings.push(format!(
                        "Schedule '{}': cron expression '{}' has {} fields (expected 5)",
                        sched.name, cron, parts.len()
                    ));
                }
            }
        }
    }

    fn check_middleware_refs(&self, warnings: &mut Vec<String>) {
        let mw_names: HashSet<&str> = self.program.middleware.iter().map(|m| m.name.as_str()).collect();

        for cmd in &self.program.commands {
            for mw in &cmd.middleware {
                if !mw_names.contains(mw.as_str()) {
                    warnings.push(format!(
                        "Command '{}' references undefined middleware '{}'",
                        cmd.name, mw
                    ));
                }
            }
            for sub in &cmd.subcommands {
                for mw in &sub.middleware {
                    if !mw_names.contains(mw.as_str()) {
                        warnings.push(format!(
                            "Subcommand '{}/{}' references undefined middleware '{}'",
                            cmd.name, sub.name, mw
                        ));
                    }
                }
            }
        }
    }

    fn check_cooldowns(&self, warnings: &mut Vec<String>) {
        let valid_scopes = ["user", "channel", "guild", "global"];
        for cmd in &self.program.commands {
            if let Some(ref cd) = cmd.cooldown {
                if let Some(ref scope) = cd.scope {
                    if !valid_scopes.contains(&scope.as_str()) {
                        warnings.push(format!(
                            "Command '{}' has invalid cooldown scope '{}'. Valid: {:?}",
                            cmd.name, scope, valid_scopes
                        ));
                    }
                }
                if cd.duration <= 0 {
                    warnings.push(format!(
                        "Command '{}' has non-positive cooldown duration ({})",
                        cmd.name, cd.duration
                    ));
                }
            }
            for sub in &cmd.subcommands {
                if let Some(ref cd) = sub.cooldown {
                    if let Some(ref scope) = cd.scope {
                        if !valid_scopes.contains(&scope.as_str()) {
                            warnings.push(format!(
                                "Subcommand '{}/{}' has invalid cooldown scope '{}'. Valid: {:?}",
                                cmd.name, sub.name, scope, valid_scopes
                            ));
                        }
                    }
                }
            }
        }
    }

    fn check_context_menus(&self, warnings: &mut Vec<String>) {
        for cm in &self.program.context_menus {
            if !VALID_CONTEXT_MENU_KINDS.contains(&cm.kind.as_str()) {
                warnings.push(format!(
                    "Context menu '{}' has invalid kind '{}'. Valid: {:?}",
                    cm.name, cm.kind, VALID_CONTEXT_MENU_KINDS
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn check(input: &str) -> Vec<String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        let checker = Checker::new(&program);
        checker.check()
    }

    #[test]
    fn test_no_warnings() {
        let warnings = check(r#"
cmd ping {
    slash true
    reply "pong"
}
"#);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_duplicate_command() {
        let warnings = check(r#"
cmd foo { slash true reply "a" }
cmd foo { slash true reply "b" }
"#);
        assert!(warnings.iter().any(|w| w.contains("Duplicate")));
    }

    #[test]
    fn test_no_trigger() {
        let warnings = check(r#"
cmd foo {
    reply "no trigger"
}
"#);
        assert!(warnings.iter().any(|w| w.contains("no slash")));
    }

    #[test]
    fn test_required_with_default() {
        let warnings = check(r#"
cmd foo {
    slash true
    param x {
        type string
        required true
        default "val"
    }
    reply x
}
"#);
        assert!(warnings.iter().any(|w| w.contains("required but has a default")));
    }

    #[test]
    fn test_prefix_trigger_no_warning() {
        let warnings = check(r#"
cmd foo {
    prefix "!"
    reply "ok"
}
"#);
        assert!(!warnings.iter().any(|w| w.contains("no slash")));
    }

    #[test]
    fn test_invalid_log_level() {
        let warnings = check(r#"
config {
    log_level "invalid"
}
cmd ping { slash true reply "pong" }
"#);
        assert!(warnings.iter().any(|w| w.contains("Invalid log level")));
    }

    #[test]
    fn test_valid_log_level() {
        let warnings = check(r#"
config {
    log_level "debug"
}
cmd ping { slash true reply "pong" }
"#);
        assert!(!warnings.iter().any(|w| w.contains("Invalid log level")));
    }

    #[test]
    fn test_invalid_intent() {
        let warnings = check(r#"
config {
    intents ["guilds", "nonexistent"]
}
cmd ping { slash true reply "pong" }
"#);
        assert!(warnings.iter().any(|w| w.contains("Unknown intent")));
    }

    #[test]
    fn test_invalid_cooldown_scope() {
        let warnings = check(r#"
cmd foo {
    slash true
    cooldown 5 "s" per planet
    reply "hi"
}
"#);
        assert!(warnings.iter().any(|w| w.contains("invalid cooldown scope")));
    }

    #[test]
    fn test_undefined_middleware() {
        let warnings = check(r#"
cmd foo {
    slash true
    middleware ["auth"]
    reply "hi"
}
"#);
        assert!(warnings.iter().any(|w| w.contains("undefined middleware")));
    }

    #[test]
    fn test_defined_middleware_no_warning() {
        let warnings = check(r#"
middleware auth {
    next
}
cmd foo {
    slash true
    middleware ["auth"]
    reply "hi"
}
"#);
        assert!(!warnings.iter().any(|w| w.contains("undefined middleware")));
    }

    #[test]
    fn test_invalid_cron_expression() {
        let warnings = check(r#"
schedule bad_cron {
    cron "* * *"
    event "message"
}
cmd ping { slash true reply "pong" }
"#);
        assert!(warnings.iter().any(|w| w.contains("cron expression")));
    }

    #[test]
    fn test_valid_cron_expression() {
        let warnings = check(r#"
schedule daily {
    cron "0 0 * * *"
    event "message"
}
cmd ping { slash true reply "pong" }
"#);
        assert!(!warnings.iter().any(|w| w.contains("cron expression")));
    }

    #[test]
    fn test_security_path_traversal() {
        let warnings = check(r#"
config {
    allowed_paths ["../../etc"]
}
cmd ping { slash true reply "pong" }
"#);
        assert!(warnings.iter().any(|w| w.contains("security risk")));
    }

    #[test]
    fn test_invalid_context_menu_kind() {
        let warnings = check(r#"
menu invalid "Bad" {
    reply "hi"
}
"#);
        assert!(warnings.iter().any(|w| w.contains("invalid kind")));
    }

    #[test]
    fn test_valid_context_menu_kind() {
        let warnings = check(r#"
menu user "UserInfo" {
    reply "user info"
}
"#);
        assert!(!warnings.iter().any(|w| w.contains("invalid kind")));
    }
}
