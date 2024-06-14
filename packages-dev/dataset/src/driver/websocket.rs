// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2025 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::*;
use bincode::{Encode, Decode};

#[derive(Debug)]
pub struct WebsocketDatasetConfig {
    url: String,
}

pub struct WebsocketDataset {
    config: WebsocketDatasetConfig,
}

impl WebsocketDataset {
    pub fn new(config: WebsocketDatasetConfig) -> Self {
        WebsocketDataset { config }
    }

    pub async fn request<M>(&self, request: WebsocketClientRequest<M>) -> Result<WebsocketServerResponse<M>>
    where
        M: DatasetModel<Self> + Encode + Decode
    {
        let msg = bincode::encode_to_vec(
            request,
            bincode::config::standard()
        ).unwrap();

        todo!()
    }
}

impl Dataset for WebsocketDataset {}

impl DatasetIndirect for WebsocketDataset {
    fn is_connected(&self) -> bool {
        todo!()
    }

    async fn connect(&mut self) -> Result<()> {
        todo!()
    }

    async fn disconnect(&mut self) -> Result<()> {
        todo!()
    }
}
