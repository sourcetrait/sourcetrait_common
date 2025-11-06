use crate::*;

/// Error nouns representing the categories of commands that this crate runs
/// internally.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdKind {
    Copy,
    Open,
    Editor,
    LaunchCtl,
    Which,
}

/// Error nouns representing the types of checks performed on results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdCheck {
    Utf8,
}

/// Represents the result of a [Command].
/// - [Self::Output]: Executed against the current process with a final result.
/// - [Self::Child]: Spawned as a child process with a handle to the child.
pub enum CommandReturn {
    Output(process::Output),
    Child(process::Child),
}

impl CommandReturn {
    /// Execute the command, checking for success.
    pub fn exec(kind: CmdKind, mut command: Command) -> BridgeResult<Self> {
        let output = command.output()
            .map_err(|source| BridgeError::cmd_call(kind, source))?;
        
        match output.status.success() {
            true => Ok(Self::Output(output)),
            false => BridgeError::err_cmd(kind, output),
        }
    }
    
    /// Execute the command without checking for success.
    pub fn exec_unchecked(kind: CmdKind, mut command: Command) -> BridgeResult<Self> {
        command.output()
            .map(|output| Self::Output(output))
            .map_err(|source| BridgeError::cmd_call(kind, source))
    }
    
    /// Execute the command, expecting an optional coerced UTF8 string on success.
    pub fn exec_for_utf8_opt(kind: CmdKind, mut command: Command) -> BridgeResult<Option<String>> {
        let mut output = command.output()
            .map_err(|source| BridgeError::cmd_call(kind, source))?;
        
        if !output.status.success() {
            return BridgeError::err_cmd::<Option<String>>(kind, output);
        }
        
        let stdout: Vec<_> = output.stdout.drain(..).collect();
        let s = String::from_utf8(stdout)
            .map(|s| s.trim().to_string())
            .map_err(|_| BridgeError::cmd_checked(kind, CmdCheck::Utf8, output))?;
        
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(s))
        }
    }
    
    /// Run the command either by [Self::exec] or [Self::spawn], as specified by
    /// the [child_process] paramter. Check for success if executed.
    pub fn run(kind: CmdKind, command: Command, child_process: bool) -> BridgeResult<Self> {
        if child_process {
            Self::spawn(kind, command)
        } else {
            Self::exec(kind, command)
        }
    }
    
    /// Run the command either by [Self::exec] or [Self::spawn], as specified by
    /// the [child_process] paramter. Do not check for success if executed.
    pub fn run_unchecked(kind: CmdKind, command: Command, child_process: bool) -> BridgeResult<Self> {
        if child_process {
            Self::spawn(kind, command)
        } else {
            Self::exec_unchecked(kind, command)
        }
    }
    
    /// Spawn the command as a child process.
    pub fn spawn(kind: CmdKind, mut command: Command) -> BridgeResult<Self> {
        command.spawn()
            .map(|child| Self::Child(child))
            .map_err(|source| BridgeError::cmd_call(kind, source))
    }
    
    pub fn is_child_process(self) -> bool {
        match self {
            Self::Child(_) => true,
            _ => false,
        }
    }

    pub fn take_child(self) -> Option<process::Child> {
        match self {
            Self::Child(child) => Some(child),
            _ => None,
        }
    }

    pub fn take_output(self) -> Option<process::Output> {
        match self {
            Self::Output(output) => Some(output),
            _ => None,
        }
    }
}
