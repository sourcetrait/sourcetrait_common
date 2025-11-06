use crate::*;

pub trait UiComponentLookup: Sized {
    fn lookup_has_command_line(&self) -> BridgeResult<bool>;
    
    fn lookup_has_graphical(&self) -> BridgeResult<bool>;
}
