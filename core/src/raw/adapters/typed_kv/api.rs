// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

use std::fmt::Debug;
use std::mem::size_of;

use async_trait::async_trait;
use chrono::Utc;

use crate::Buffer;
use crate::EntryMode;
use crate::Error;
use crate::ErrorKind;
use crate::Metadata;
use crate::Result;
use crate::Scheme;

/// Adapter is the typed adapter to underlying kv services.
///
/// By implement this trait, any kv service can work as an OpenDAL Service.
///
/// # Notes
///
/// `typed_kv::Adapter` is the typed version of `kv::Adapter`. It's more
/// efficient if the underlying kv service can store data with its type. For
/// example, we can store `Bytes` along with it's metadata so that we don't
/// need to serialize/deserialize it when we get it from the service.
///
/// Ideally, we should use `typed_kv::Adapter` instead of `kv::Adapter` for
/// in-memory rust libs like moka and dashmap.
#[async_trait]
pub trait Adapter: Send + Sync + Debug + Unpin + 'static {
    /// Get the scheme and name of current adapter.
    fn info(&self) -> Info;

    /// Get a value from adapter.
    async fn get(&self, path: &str) -> Result<Option<Value>>;

    /// Get a value from adapter.
    fn blocking_get(&self, path: &str) -> Result<Option<Value>>;

    /// Set a value into adapter.
    async fn set(&self, path: &str, value: Value) -> Result<()>;

    /// Set a value into adapter.
    fn blocking_set(&self, path: &str, value: Value) -> Result<()>;

    /// Delete a value from adapter.
    async fn delete(&self, path: &str) -> Result<()>;

    /// Delete a value from adapter.
    fn blocking_delete(&self, path: &str) -> Result<()>;

    /// Scan a key prefix to get all keys that start with this key.
    async fn scan(&self, path: &str) -> Result<Vec<String>> {
        let _ = path;

        Err(Error::new(
            ErrorKind::Unsupported,
            "typed_kv adapter doesn't support this operation",
        )
        .with_operation("typed_kv::Adapter::scan"))
    }

    /// Scan a key prefix to get all keys that start with this key
    /// in blocking way.
    fn blocking_scan(&self, path: &str) -> Result<Vec<String>> {
        let _ = path;

        Err(Error::new(
            ErrorKind::Unsupported,
            "typed_kv adapter doesn't support this operation",
        )
        .with_operation("typed_kv::Adapter::blocking_scan"))
    }
}

/// Value is the typed value stored in adapter.
///
/// It's cheap to clone so that users can read data without extra copy.
#[derive(Debug, Clone)]
pub struct Value {
    /// Metadata of this value.
    pub metadata: Metadata,
    /// The corresponding content of this value.
    pub value: Buffer,
}

impl Value {
    /// Create a new dir of value.
    pub fn new_dir() -> Self {
        Self {
            metadata: Metadata::new(EntryMode::DIR)
                .with_content_length(0)
                .with_last_modified(Utc::now()),
            value: Buffer::new(),
        }
    }

    /// Size returns the in-memory size of Value.
    pub fn size(&self) -> usize {
        size_of::<Metadata>() + self.value.len()
    }
}

/// Capability is used to describe what operations are supported
/// by Typed KV Operator.
#[derive(Copy, Clone, Default)]
pub struct Capability {
    /// If typed_kv operator supports get natively.
    pub get: bool,
    /// If typed_kv operator supports set natively.
    pub set: bool,
    /// If typed_kv operator supports delete natively.
    pub delete: bool,
    /// If typed_kv operator supports scan natively.
    pub scan: bool,
}

impl Debug for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = vec![];

        if self.get {
            s.push("Get")
        }
        if self.set {
            s.push("Set");
        }
        if self.delete {
            s.push("Delete");
        }
        if self.scan {
            s.push("Scan");
        }

        write!(f, "{{ {} }}", s.join(" | "))
    }
}

/// Info for this key value accessor.
pub struct Info {
    scheme: Scheme,
    name: String,
    capabilities: Capability,
}

impl Info {
    /// Create a new KeyValueAccessorInfo.
    pub fn new(scheme: Scheme, name: &str, capabilities: Capability) -> Self {
        Self {
            scheme,
            name: name.to_string(),
            capabilities,
        }
    }

    /// Get the scheme.
    pub fn scheme(&self) -> Scheme {
        self.scheme
    }

    /// Get the name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the capabilities.
    pub fn capabilities(&self) -> Capability {
        self.capabilities
    }
}
