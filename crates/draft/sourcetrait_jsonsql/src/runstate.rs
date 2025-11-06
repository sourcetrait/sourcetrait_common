use crate::*;

#[derive(Debug, Clone)]
pub(crate) struct RunState {
    pub(crate) cli: Cli,
}

impl RunState {
    pub(crate) fn new(cli: Cli) -> Self {
        Self { cli }
    }
}