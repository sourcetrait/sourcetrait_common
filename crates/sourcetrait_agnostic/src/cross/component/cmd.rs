use crate::*;

pub trait CmdComponentTrait {
    /// Runs the appropriate command to interactively open a file if a GUI
    /// session is available. Fails if GUI is not available.
    /// - Linux + XDG: `xdg-open <FILE>`
    /// - MacOS: `open <FILE>`
    /// - Windows: `start <FILE>`
    fn open<P>(&self, path: P, child_process: bool) -> CrossResult<CommandReturn>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    /// Runs the appropriate command to interactively open a file editor for
    /// the specified file.
    /// 
    /// - If a GUI session is available:
    ///   - Linux + XDG: `xdg-open <FILE>`
    ///   - MacOS: `open <FILE>`
    ///   - Windows: `start <FILE>`
    /// - If a purely terminal session is available:
    ///   - Uses the environment's EDITOR setting, if available.
    ///   - Otherwise, runs a handful of guesses for common terminal editors.
    ///   - Fails if nothing is found. 
    fn open_with_editor<P>(&self, file: P, child_process: bool) -> CrossResult<CommandReturn>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    /// Finds the the canonical path for the command specified, using the
    /// environment's PATH, in order.
    fn which(&self, cmd: &str) -> CrossResult<Option<Command>>;
    
    /// Finds the first canonical path found for the commands specified,
    /// using the environment's PATH, in order.
    /// 
    /// Intended for guessing which command to use.
    fn guess_which(&self, from_where: &Vec<GuessWhere<'_>>, cmd: &str) -> CrossResult<Option<Command>>;
    
    fn guess_which_of(&self, from_where: &Vec<GuessWhere<'_>>, cmds: Vec<&str>) -> CrossResult<Option<Command>>;
}

#[allow(private_bounds)]
pub struct StandardCmdComponent<LOOKUP: CmdComponentLookup>(pub(crate) LOOKUP);

#[allow(private_bounds)]
impl<LOOKUP: CmdComponentLookup> StandardCmdComponent<LOOKUP> {
    fn lookup(&self) -> &LOOKUP { &self.0 }
}

impl<LOOKUP: CmdComponentLookup> CmdComponentTrait for StandardCmdComponent<LOOKUP> {
    fn open<P>(&self, path: P, child_process: bool) -> CrossResult<CommandReturn>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        let cmd = self.lookup().lookup_gui_open_command(path)?;
        let ret = CommandReturn::run(CmdKind::Open, cmd, child_process)?;
        Ok(ret)
    }

    fn open_with_editor<P>(&self, file: P, child_process: bool) -> CrossResult<CommandReturn>
    where
        P: AsRef<Path> + Into<PathBuf>
    {
        let editor_cmd = match crate::PLATFORM.ui().preference()? {
            UserInterface::CLI => {
                let guesses = self.lookup().lookup_guess_cli_editor_open_command(file)?;
                let guess = self.guess_which_of(&guesses.wherein, guesses.which)?
                    .ok_or_else(|| CrossError::not_found(CrossErr::Editor))?;
                Ok(guess)
            },
            UserInterface::GUI => self.lookup().lookup_gui_editor_open_command(file),
            UserInterface::None => return CrossError::err_not_found(CrossErr::Editor),
        }?;
        
        let ret = CommandReturn::run(CmdKind::Editor, editor_cmd, child_process)?;
        Ok(ret)
    }

    fn which(&self, cmd: &str) -> CrossResult<Option<Command>> {
        for env_path in crate::PLATFORM.path().env_paths()?.iter() {
            let filepath = env_path.join(cmd);
            if filepath.is_file() {
                return Ok(Some(Command::new(filepath)))
            }
        }
        
        Ok(None)
    }

    fn guess_which(&self, from_where: &Vec<GuessWhere<'_>>, cmd: &str) -> CrossResult<Option<Command>> {
        for from in from_where {
            let found = match from {
                GuessWhere::EnvVar(varname) => match env::var(varname) {
                    Ok(value) => Some(Command::new(value)),
                    _ => None,
                },
                GuessWhere::EnvPaths => match self.which(cmd.as_ref()) {
                    Ok(Some(cmd)) => Some(cmd),
                    _ => None,
                },
            };
            
            if found.is_some() {
                return Ok(found);
            }
        }
        
        Ok(None)
    }
    
    
    fn guess_which_of(&self, from_where: &Vec<GuessWhere<'_>>, cmds: Vec<&str>) -> CrossResult<Option<Command>> {
        for cmd in cmds {
            if let Some(found) = self.guess_which(from_where, cmd)? {
                return Ok(Some(found))
            }
        }
        
        Ok(None)
    }
}
