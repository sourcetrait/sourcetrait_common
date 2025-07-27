use std::{convert::Infallible, marker::PhantomData, path::PathBuf};
use thiserror;
use crate::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    //#[error("{0}")]
    //Io(#[from] std::io::Error)
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError<R: 'static + DotRepoType> {
    #[error("{0}")]
    Io(String, #[source] std::io::Error),
    #[error("{r} repository designator `Top` not found behind path: {0:?}", r = R::DEFINITION.subdir)]
    Topless(PathBuf),
    #[error("{r} repository designator `Top` already exists behind path: {0:?}", r = R::DEFINITION.subdir)]
    TopAlreadyExists(PathBuf),
    #[error("Unspecified error for repository type: {}", R::DEFINITION.subdir)]
    _Unspecified(Infallible, PhantomData<R>),
}

pub type RepoResult<R, T> = std::result::Result<T, RepoError<R>>;