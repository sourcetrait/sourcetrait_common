use crate::*;

pub(crate) fn _expand_env(s: &str) -> CrossResult<Cow<'_, str>> {
    shellexpand::env(s)
        .map_err(|source| CrossError::env_var(source.var_name, source.cause))
}
