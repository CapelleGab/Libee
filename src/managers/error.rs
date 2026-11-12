use thiserror::Error;

#[derive(Debug, Error)]
pub enum ManagerError {
    #[error("`{0}` is not installed")]
    NotInstalled(String),

    #[error("`{command}` exited with {}: {stderr}", code.map_or("a signal".to_owned(), |code| format!("code {code}")))]
    CommandFailed {
        command: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("unexpected output from `{command}`: {reason}")]
    UnexpectedOutput { command: String, reason: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::ManagerError;

    #[test]
    fn describes_failed_commands() {
        let error = ManagerError::CommandFailed {
            command: "npm ls -g".to_owned(),
            code: Some(1),
            stderr: "EACCES".to_owned(),
        };

        assert_eq!(error.to_string(), "`npm ls -g` exited with code 1: EACCES");
    }
}
