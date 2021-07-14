use std::io::SeekFrom;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{error::Error, read::io::ReadSeek};

pub struct LocalHeap {
    pub signature: [u8; 4],
    pub version: u8,
    pub reserved: [u8; 3],
    pub data_segment_size: u64,
    pub offset_to_free_list: u64,
    pub address_of_data_segment: u64,
}

impl LocalHeap {
    pub fn read(input: &mut impl ReadSeek, address: u64) -> Result<LocalHeap, Error> {
        input.seek(SeekFrom::Start(address))?;
        let local_heap = LocalHeap {
            signature: {
                let mut result = [0; 4];
                input.read_exact(&mut result)?;
                result
            },
            version: input.read_u8()?,
            reserved: {
                let mut result = [0; 3];
                input.read_exact(&mut result)?;
                result
            },
            data_segment_size: input.read_u64::<LittleEndian>()?,
            offset_to_free_list: input.read_u64::<LittleEndian>()?,
            address_of_data_segment: input.read_u64::<LittleEndian>()?,
        };
        assert!(local_heap.signature == "HEAP".as_bytes());
        assert!(local_heap.version == 0);
        Ok(local_heap)
    }

    pub fn object_name(&self, input: &mut impl ReadSeek, offset: u64) -> Result<String, Error> {
        input.seek(SeekFrom::Start(self.address_of_data_segment + offset))?;
        let mut end = 0;
        while input.read_u8()? != 0 {
            end += 1;
        }
        input.seek(SeekFrom::Start(self.address_of_data_segment + offset))?;
        let mut result = vec![0; end];
        input.read_exact(&mut result)?;
        Ok(String::from_utf8(result)?)
    }
}
