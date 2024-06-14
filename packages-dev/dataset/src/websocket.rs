// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::*;
use bincode::{Encode, Decode};

#[derive(thiserror::Error, PartialEq, Debug, Encode, Decode)]
pub enum WebsocketErrorResponse {
    #[error("Not found")]
    NotFound,
    #[error("{0}")]
    Generic(String)
}

#[derive(Debug, Encode, Decode)]
pub enum WebsocketClientRequest<M>
where
    M: DatasetModel<WebsocketDataset>
{
    Get(WebsocketGetRequest),
    Put(WebsocketPutRequest<M>),
    Delete(WebsocketDeleteRequest),
}

#[derive(Debug, Encode, Decode)]
pub enum WebsocketServerResponse<M>
where
    M: DatasetModel<WebsocketDataset>
{
    Error(WebsocketErrorResponse),
    GetResponse(WebsocketGetResponse<M>),
    PutResponse(WebsocketPutResponse),
    DeleteResponse(WebsocketDeleteResponse),
}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketGetRequest {
    authorative_id: u64
}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketGetResponse<M>
where
    M: DatasetModel<WebsocketDataset>
{
    model: Option<M>
}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketPutRequest<M>
where
    M: DatasetModel<WebsocketDataset>
{
    model: M
}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketPutResponse {
    authorative_id: u64
}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketDeleteRequest{}

#[derive(Debug, Encode, Decode)]
pub struct WebsocketDeleteResponse{}
