use std::io;

use super::ManagerError;
use crate::shared::CommandSpec;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CommandOutput {
    pub success: bool,
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

/// Runs commands for adapters. Swapped for a fake in tests so adapters can be
/// checked against recorded outputs without the real managers installed.
pub trait CommandRunner: Send + Sync {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, ManagerError>;

    fn stdout(&self, command: &CommandSpec) -> Result<String, ManagerError> {
        let output = self.run(command)?;
        if output.success {
            Ok(output.stdout)
        } else {
            Err(ManagerError::CommandFailed {
                command: command.to_string(),
                code: output.code,
                stderr: output.stderr.trim().to_owned(),
            })
        }
    }
}

pub struct SystemRunner;

impl CommandRunner for SystemRunner {
    fn run(&self, command: &CommandSpec) -> Result<CommandOutput, ManagerError> {
        let output = command
            .to_command()
            .output()
            .map_err(|error| match error.kind() {
                io::ErrorKind::NotFound => ManagerError::NotInstalled(command.program().to_owned()),
                _ => ManagerError::Io(error),
            })?;

        Ok(CommandOutput {
            success: output.status.success(),
            code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
    }
}

#[cfg(test)]
pub(crate) mod fake {
    use std::collections::HashMap;

    use super::{CommandOutput, CommandRunner};
    use crate::managers::ManagerError;
    use crate::shared::CommandSpec;

    /// Answers commands from a table; unknown programs behave as not installed.
    #[derive(Default)]
    pub struct FakeRunner {
        outputs: HashMap<String, CommandOutput>,
    }

    impl FakeRunner {
        pub fn with(mut self, command: &str, stdout: &str) -> Self {
            let output = CommandOutput {
                success: true,
                code: Some(0),
                stdout: stdout.to_owned(),
                stderr: String::new(),
            };
            self.outputs.insert(command.to_owned(), output);
            self
        }

        pub fn failing(mut self, command: &str, code: i32, stderr: &str) -> Self {
            let output = CommandOutput {
                success: false,
                code: Some(code),
                stdout: String::new(),
                stderr: stderr.to_owned(),
            };
            self.outputs.insert(command.to_owned(), output);
            self
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(&self, command: &CommandSpec) -> Result<CommandOutput, ManagerError> {
            self.outputs
                .get(&command.to_string())
                .cloned()
                .ok_or_else(|| ManagerError::NotInstalled(command.program().to_owned()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeRunner;
    use super::{CommandRunner, SystemRunner};
    use crate::managers::ManagerError;
    use crate::shared::CommandSpec;

    #[test]
    fn missing_program_is_reported_as_not_installed() {
        let command = CommandSpec::new("libee-missing-program").arg("--version");

        let error = SystemRunner.run(&command).unwrap_err();

        assert!(
            matches!(error, ManagerError::NotInstalled(program) if program == "libee-missing-program")
        );
    }

    #[test]
    fn runs_a_real_program() {
        let output = SystemRunner
            .run(&CommandSpec::new("cargo").arg("--version"))
            .unwrap();

        assert!(output.success);
        assert!(output.stdout.starts_with("cargo "));
    }

    #[test]
    fn stdout_turns_failures_into_errors() {
        let runner = FakeRunner::default().failing("npm ls -g", 1, "EACCES\n");

        let error = runner
            .stdout(&CommandSpec::new("npm").args(["ls", "-g"]))
            .unwrap_err();

        assert_eq!(error.to_string(), "`npm ls -g` exited with code 1: EACCES");
    }

    #[test]
    fn stdout_returns_output_on_success() {
        let runner = FakeRunner::default().with("brew --version", "Homebrew 4.4.0\n");

        let stdout = runner
            .stdout(&CommandSpec::new("brew").arg("--version"))
            .unwrap();

        assert_eq!(stdout, "Homebrew 4.4.0\n");
    }
}
