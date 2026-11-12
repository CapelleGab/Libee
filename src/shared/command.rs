use std::fmt;
use std::process::Command;

/// A command as a program plus separate arguments, never a shell string,
/// so package names can't inject extra commands.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    program: String,
    args: Vec<String>,
}

impl CommandSpec {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn program(&self) -> &str {
        &self.program
    }

    pub fn arguments(&self) -> &[String] {
        &self.args
    }

    pub fn to_command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command
    }
}

/// Renders the command so it can be copied into a terminal as-is.
impl fmt::Display for CommandSpec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", quote(&self.program))?;
        for arg in &self.args {
            write!(f, " {}", quote(arg))?;
        }
        Ok(())
    }
}

fn quote(value: &str) -> String {
    let is_safe = !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./@=:+,".contains(c));

    if is_safe {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', r"'\''"))
    }
}

#[cfg(test)]
mod tests {
    use super::CommandSpec;

    #[test]
    fn keeps_arguments_separate() {
        let spec = CommandSpec::new("npm").args(["uninstall", "-g", "typescript; rm -rf ~"]);

        assert_eq!(spec.program(), "npm");
        assert_eq!(
            spec.arguments(),
            ["uninstall", "-g", "typescript; rm -rf ~"]
        );
    }

    #[test]
    fn displays_plain_arguments_unquoted() {
        let spec = CommandSpec::new("npm").args(["ls", "-g", "--json", "@scope/pkg"]);

        assert_eq!(spec.to_string(), "npm ls -g --json @scope/pkg");
    }

    #[test]
    fn quotes_unsafe_arguments_for_copy_paste() {
        let spec = CommandSpec::new("npm")
            .arg("uninstall")
            .arg("evil; rm -rf ~")
            .arg("it's")
            .arg("");

        assert_eq!(
            spec.to_string(),
            r#"npm uninstall 'evil; rm -rf ~' 'it'\''s' ''"#
        );
    }

    #[test]
    fn builds_a_std_command() {
        let command = CommandSpec::new("brew")
            .args(["list", "--versions"])
            .to_command();

        assert_eq!(command.get_program(), "brew");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["list", "--versions"]
        );
    }
}
