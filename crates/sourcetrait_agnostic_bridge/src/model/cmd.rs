//use crate::*;

pub enum GuessWhere<'a> {
    EnvVar(&'a str),
    EnvPaths,
}

pub struct PathGuess<'a> {
    /// where to search for each guess
    pub wherein: Vec<GuessWhere<'a>>,
    /// names of commands to search for
    pub which: Vec<&'a str>,
}

