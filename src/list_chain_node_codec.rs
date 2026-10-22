use crate::{list_chain_view_storage::ListChainNode, storage_errors::StorageError};

pub struct ListChainNodeCodec;

impl ListChainNodeCodec {
    pub const PREV_OFFSET: usize = 8;
    pub const NODE_OFFSET: usize = 8;
    pub const NEXT_OFFSET: usize = 8;
    pub const RECORD_SIZE: usize = Self::PREV_OFFSET + Self::NODE_OFFSET + Self::NEXT_OFFSET;

    pub fn encode_record(node: &ListChainNode) -> [u8; Self::RECORD_SIZE] {
        let mut bytes = [0; Self::RECORD_SIZE];
        bytes[0..Self::PREV_OFFSET].copy_from_slice(&node.prev_offset.to_le_bytes());
        bytes[Self::PREV_OFFSET..Self::PREV_OFFSET + Self::NODE_OFFSET]
            .copy_from_slice(&node.node_offset.to_le_bytes());
        bytes[Self::PREV_OFFSET + Self::NODE_OFFSET..Self::RECORD_SIZE]
            .copy_from_slice(&node.next_offset.to_le_bytes());
        bytes
    }

    pub fn decode_record(bytes: &[u8]) -> Result<ListChainNode, StorageError> {
        if bytes.len() != Self::RECORD_SIZE {
            return Err(StorageError::ListChainCorruptedRecord(
                "Expected a complete list chain record".to_string(),
            ));
        }

        let prev_offset = u64::from_le_bytes(
            bytes[0..Self::PREV_OFFSET].try_into().map_err(|_| {
                StorageError::ListChainCorruptedRecord("Corrupted prev_offset".to_string())
            })?,
        );
        let node_offset = u64::from_le_bytes(
            bytes[Self::PREV_OFFSET..Self::PREV_OFFSET + Self::NODE_OFFSET]
                .try_into()
                .map_err(|_| {
                    StorageError::ListChainCorruptedRecord("Corrupted node_offset".to_string())
                })?,
        );
        let next_offset = u64::from_le_bytes(
            bytes[Self::PREV_OFFSET + Self::NODE_OFFSET..Self::RECORD_SIZE]
                .try_into()
                .map_err(|_| {
                    StorageError::ListChainCorruptedRecord("Corrupted next_offset".to_string())
                })?,
        );

        Ok(ListChainNode {
            prev_offset,
            node_offset,
            next_offset,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_and_decodes_node_offsets_as_little_endian() {
        let node = ListChainNode {
            prev_offset: 1,
            node_offset: u64::MAX,
            next_offset: 0x0102_0304_0506_0708,
        };
        let buffer = ListChainNodeCodec::encode_record(&node);

        let decoded = ListChainNodeCodec::decode_record(&buffer).unwrap();

        assert_eq!(&buffer[0..8], &node.prev_offset.to_le_bytes());
        assert_eq!(&buffer[8..16], &node.node_offset.to_le_bytes());
        assert_eq!(&buffer[16..24], &node.next_offset.to_le_bytes());
        assert_eq!(decoded, node);
    }

    #[test]
    fn rejects_incomplete_or_oversized_records() {
        for length in [0, 7, 8, 15, 16, 23, 25] {
            let error = ListChainNodeCodec::decode_record(&vec![0; length]).unwrap_err();

            assert!(matches!(error, StorageError::ListChainCorruptedRecord(_)));
        }
    }
}
