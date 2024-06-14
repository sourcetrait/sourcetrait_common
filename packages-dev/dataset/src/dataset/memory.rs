// Asmov Common Dataset: Library for application data modeling between clients and servers
// Copyright (C) 2024 Asmov LLC
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::*;

/// A dataset that is stored in memory using hashmaps with ID as the key.
///
/// Each model's hashmap should provide a `fn iter_mymodels() -> Iter<ID, MyModel>` for direct access.
pub trait MemoryDataset: DatasetDirect + Default {}
