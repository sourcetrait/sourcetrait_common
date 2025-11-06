use crate::*;

pub trait CmdComponentLookup {
    fn lookup_guess_cli_editor_open_command<P>(&self, filepath: P) -> BridgeResult<PathGuess<'_>>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    fn lookup_gui_editor_open_command<P>(&self, filepath: P) -> BridgeResult<Command>
    where
        P: AsRef<Path> + Into<PathBuf>;
    
    fn lookup_gui_open_command<P>(&self, filepath: P) -> BridgeResult<Command>
    where
        P: AsRef<Path> + Into<PathBuf>;
}
