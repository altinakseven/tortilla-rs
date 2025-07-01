// src/taqueria.rs

use alkanes_runtime::storage::StoragePointer;
use alkanes_support::id::AlkaneId;
use anyhow::Result;
use metashrew_support::index_pointer::KeyValuePointer;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DeserializeAs, SerializeAs};

pub struct AlkaneIdAsBytes;

impl SerializeAs<AlkaneId> for AlkaneIdAsBytes {
    fn serialize_as<S>(source: &AlkaneId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let bytes: Vec<u8> = source.clone().into();
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> DeserializeAs<'de, AlkaneId> for AlkaneIdAsBytes {
    fn deserialize_as<D>(deserializer: D) -> Result<AlkaneId, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: &[u8] = serde::Deserialize::deserialize(deserializer)?;
        AlkaneId::parse(&mut std::io::Cursor::new(bytes.to_vec()))
            .map_err(serde::de::Error::custom)
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct TaqueriaNode {
    #[serde_as(as = "Vec<AlkaneIdAsBytes>")]
    pub tortillas: Vec<AlkaneId>,
    #[serde_as(as = "Option<AlkaneIdAsBytes>")]
    pub left: Option<AlkaneId>,
    #[serde_as(as = "Option<AlkaneIdAsBytes>")]
    pub right: Option<AlkaneId>,
}

pub struct TaqueriaManager {
    pub root_pointer: StoragePointer,
}

impl TaqueriaManager {
    pub fn new(base_pointer: StoragePointer) -> Self {
        Self {
            root_pointer: base_pointer.select(&b"/root".to_vec()),
        }
    }

    fn get_node_pointer(taqueria_id: &AlkaneId) -> StoragePointer {
        let id_vec: Vec<u8> = taqueria_id.clone().into();
        StoragePointer::from_keyword("/taquerias/").select(&id_vec)
    }

    fn get_node(taqueria_id: &AlkaneId) -> Result<Option<TaqueriaNode>> {
        let pointer = Self::get_node_pointer(taqueria_id);
        let data = pointer.get();
        if data.is_empty() {
            return Ok(None);
        }
        Ok(Some(ciborium::from_reader(&data[..])?))
    }

    fn set_node(taqueria_id: &AlkaneId, node: &TaqueriaNode) -> Result<()> {
        let mut pointer = Self::get_node_pointer(taqueria_id);
        let mut buffer = Vec::new();
        ciborium::into_writer(node, &mut buffer)?;
        pointer.set(buffer.into());
        Ok(())
    }

    pub fn add_tortilla_to_taqueria(
        &self,
        taqueria_id: &AlkaneId,
        tortilla_id: &AlkaneId,
    ) -> Result<()> {
        let mut root_id = self.get_root()?;
        self.insert(&mut root_id, taqueria_id, tortilla_id)?;
        self.set_root(root_id)
    }

    fn get_root(&self) -> Result<Option<AlkaneId>> {
        let root_data = self.root_pointer.get();
        if root_data.is_empty() {
            Ok(None)
        } else {
            Ok(Some(
                AlkaneId::parse(&mut std::io::Cursor::new(root_data.to_vec()))
                    .map_err(|e| anyhow::anyhow!(e))?,
            ))
        }
    }

    fn set_root(&mut self, root_id: Option<AlkaneId>) -> Result<()> {
        if let Some(id) = root_id {
            let id_vec: Vec<u8> = id.into();
            self.root_pointer.set(id_vec.into());
        } else {
            self.root_pointer.set(vec![].into());
        }
        Ok(())
    }

    fn insert(&self, node_id: &mut Option<AlkaneId>, taqueria_id: &AlkaneId, tortilla_id: &AlkaneId) -> Result<()> {
        match node_id {
            Some(id) => {
                let mut node = Self::get_node(id)?.unwrap_or_else(|| TaqueriaNode {
                    tortillas: vec![],
                    left: None,
                    right: None,
                });
                if taqueria_id < id {
                    self.insert(&mut node.left, taqueria_id, tortilla_id)?;
                } else if taqueria_id > id {
                    self.insert(&mut node.right, taqueria_id, tortilla_id)?;
                } else {
                    if !node.tortillas.contains(tortilla_id) {
                        node.tortillas.push(tortilla_id.clone());
                    }
                }
                Self::set_node(id, &node)?;
            }
            None => {
                let new_node = TaqueriaNode {
                    tortillas: vec![tortilla_id.clone()],
                    left: None,
                    right: None,
                };
                Self::set_node(taqueria_id, &new_node)?;
                *node_id = Some(taqueria_id.clone());
            }
        }
        Ok(())
    }

    pub fn get_tortillas_for_taqueria(&self, taqueria_id: &AlkaneId) -> Result<Option<Vec<AlkaneId>>> {
        let mut current_id = self.get_root()?;
        while let Some(id) = current_id {
            let node = Self::get_node(&id)?;
            if let Some(n) = node {
                if taqueria_id < &id {
                    current_id = n.left;
                } else if taqueria_id > &id {
                    current_id = n.right;
                } else {
                    return Ok(Some(n.tortillas));
                }
            } else {
                return Ok(None);
            }
        }
        Ok(None)
    }
}