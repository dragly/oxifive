use crate::error::Error;
use crate::read::{
    data_object::DataObject,
    data_storage::DataStorage,
    dataspace::Dataspace,
    datatype::Datatype,
    datatype::DatatypeEncoding,
    file::FileReader,
    filter_pipeline::{FilterPipeline, FilterType},
    node::{parse_node, BTreeNode},
};
use ndarray::{Array, ArrayD, Dimension, IxDyn, SliceInfo, SliceInfoElem};
use num_traits::identities::Zero;
use std::{
    collections::HashMap,
    fmt::Debug,
    io::{Cursor, Read, Seek, SeekFrom},
};

#[derive(Clone, Debug)]
pub struct Dataset {
    pub data_object: DataObject,
}

pub trait DatatypeVerifiable {
    fn verify(datatype: &Datatype) -> Result<(), Error>;
}

macro_rules! add_verifiable_type {
    ($rust_type:ty, $encoding:expr, $size:expr) => {
        impl DatatypeVerifiable for $rust_type {
            fn verify(datatype: &Datatype) -> Result<(), Error> {
                if datatype.encoding != $encoding || datatype.size != $size {
                    return Err(Error::OxifiveError(format!(
                        "Wrong datatype found for {}: {:?}",
                        stringify!($rust_type),
                        datatype
                    )));
                }
                Ok(())
            }
        }
    };
}

add_verifiable_type!(f32, DatatypeEncoding::FloatingPoint, 4);
add_verifiable_type!(f64, DatatypeEncoding::FloatingPoint, 8);
add_verifiable_type!(u8, DatatypeEncoding::FixedPoint, 1);

impl Dataset {
    pub fn shape(&self) -> Vec<u64> {
        self.data_object.dataspaces[0].shape.clone()
    }

    pub fn datatype(&self) -> Datatype {
        self.data_object.datatypes[0].clone()
    }

    pub fn read<T, D>(&self, file: &mut FileReader) -> Result<Array<T, D>, Error>
    where
        T: Clone + Copy + Debug + DatatypeVerifiable + Zero,
        D: Dimension,
    {
        let data_object = &self.data_object;
        let datatype = data_object.datatypes[0].clone();
        log::info!("Datatype {:#?}", datatype);
        let dataspace = data_object.dataspaces[0].clone();
        log::info!("Shape {:#?}", dataspace.shape);
        let data = data_object.data[0].clone();
        match data {
            DataStorage::Chunked {
                chunk_shape,
                address,
            } => self.read_chunked(
                file,
                &chunk_shape,
                address,
                &datatype,
                &dataspace,
                &data_object.filter_pipelines,
            ),
            DataStorage::Contiguous { address, size } => {
                assert!(data_object.filter_pipelines.is_empty());
                self.read_contiguous(file, address, size, &datatype, &dataspace)
            }
        }
    }

    fn read_contiguous<T, D>(
        &self,
        file: &mut FileReader,
        address: u64,
        size: u64,
        datatype: &Datatype,
        dataspace: &Dataspace,
    ) -> Result<Array<T, D>, Error>
    where
        T: Clone + Copy + Debug + DatatypeVerifiable + Zero,
        D: Dimension,
    {
        T::verify(datatype)?;

        let mut buffer = vec![0; size as usize];
        file.input.seek(SeekFrom::Start(address))?;
        file.input.read_exact(&mut buffer)?;

        let mut clone = std::mem::ManuallyDrop::new(buffer);
        let vector = unsafe {
            Vec::from_raw_parts(
                clone.as_mut_ptr() as *mut T,
                clone.len() / std::mem::size_of::<T>(),
                clone.capacity() / std::mem::size_of::<T>(),
            )
        };

        log::info!(
            "Contiguous len {:?} and shape {:?}",
            vector.len(),
            dataspace.shape
        );

        let shape: Vec<usize> = dataspace.shape.iter().map(|&x| x as usize).collect();
        let array = ArrayD::from_shape_vec(shape, vector)?;
        Ok(array.into_dimensionality()?)
    }

    fn read_chunked<T, D>(
        &self,
        file: &mut FileReader,
        chunk_shape: &[u32],
        address: u64,
        datatype: &Datatype,
        dataspace: &Dataspace,
        filter_pipelines: &[FilterPipeline],
    ) -> Result<Array<T, D>, Error>
    where
        T: Clone + Copy + Debug + DatatypeVerifiable + Zero,
        D: Dimension,
    {
        T::verify(datatype)?;

        log::info!("Data chunk shape {:#?}", chunk_shape);

        let dimensions = chunk_shape.len();

        let root_node = parse_node(&mut file.input, address, dimensions)?;

        let mut nodes = HashMap::<u8, Vec<BTreeNode>>::new();
        let mut node_level = root_node.node_level;
        nodes.insert(node_level, vec![root_node]);
        while node_level != 0 {
            let mut next_nodes = vec![];
            for parent_node in &nodes[&node_level] {
                for key in &parent_node.keys {
                    let address = key.chunk_address;
                    next_nodes.push(parse_node(&mut file.input, address, dimensions)?);
                }
            }
            let next_node_level = next_nodes[0].node_level;
            nodes.insert(next_node_level, next_nodes);
            node_level = next_node_level;
        }

        log::info!("Nodes {:#?}", nodes);

        // TODO make into u64 for safety
        let element_count: usize = chunk_shape.iter().product::<u32>() as usize;

        log::info!("Element count {}", element_count);

        let shape: Vec<usize> = dataspace.shape.iter().map(|&x| x as usize).collect();
        // TODO verify D here
        //assert!(shape.len() == D::dimensions());
        let mut array = ArrayD::<T>::zeros(shape);
        let item_size = datatype.size as usize;
        let chunk_buffer_size = element_count * item_size;

        for filter in filter_pipelines {
            log::info!("Found filter {:#?}", filter);
        }
        for node in &nodes[&0] {
            for node_key in &node.keys {
                let address = node_key.chunk_address;
                file.input.seek(SeekFrom::Start(address))?;
                let chunk_array = {
                    let byte_buffer = {
                        if filter_pipelines.is_empty() {
                            // TODO compare with chunk_size
                            // TODO might be untested
                            let mut buffer = vec![0; chunk_buffer_size];
                            file.input.read_exact(&mut buffer)?;
                            buffer
                        } else {
                            let mut buffer = vec![0; node_key.chunk_size as usize];
                            file.input.read_exact(&mut buffer)?;
                            if node_key.filter_mask != 0 {
                                return Err(Error::OxifiveError(format!(
                                    "Filter masks are not yet supported: {}",
                                    node_key.filter_mask
                                )));
                            }
                            for filter in filter_pipelines.iter().rev() {
                                log::info!("Running filter {:#?}", filter);
                                match filter.filter_type {
                                    FilterType::ShuffleFilter => {
                                        // TODO consider using itertools::interleave
                                        let buffer_size = buffer.len();
                                        let mut unshuffled_buffer = vec![0; buffer_size];
                                        let item_count = buffer_size / item_size;
                                        for item_index in 0..item_count {
                                            for byte_index in 0..item_size {
                                                let unshuffled_index =
                                                    item_index * item_size + byte_index;
                                                let shuffled_index =
                                                    byte_index * item_count + item_index;
                                                unshuffled_buffer[unshuffled_index] =
                                                    buffer[shuffled_index];
                                            }
                                        }
                                        buffer.copy_from_slice(&unshuffled_buffer[..]);
                                    }
                                    FilterType::GzipDeflateFilter => {
                                        let mut reader = Cursor::new(&buffer);
                                        let mut decoder =
                                            flate2::read::ZlibDecoder::new(&mut reader);
                                        let mut decompressed = vec![];
                                        decoder.read_to_end(&mut decompressed)?;
                                        log::info!("Decompressed into {}", decompressed.len());
                                        //let decompressed = miniz_oxide::inflate::decompress_to_vec(&buffer)?;
                                        buffer.resize(decompressed.len(), 0);
                                        buffer.clone_from_slice(&decompressed[..]);
                                    }
                                    _ => {
                                        return Err(Error::OxifiveError(format!(
                                            "Unsupported filter type: {:#?}",
                                            filter
                                        )));
                                    }
                                }
                            }
                            buffer
                        }
                    };
                    let mut clone = std::mem::ManuallyDrop::new(byte_buffer);
                    let chunk_vector = unsafe {
                        Vec::from_raw_parts(
                            clone.as_mut_ptr() as *mut T,
                            clone.len() / std::mem::size_of::<T>(),
                            clone.capacity() / std::mem::size_of::<T>(),
                        )
                    };
                    let shape: Vec<usize> = chunk_shape[..chunk_shape.len() - 1]
                        .iter()
                        .map(|&x| x as usize)
                        .collect();
                    log::info!(
                        "Reading vector of length {} into shape {:?}",
                        chunk_vector.len(),
                        shape
                    );
                    ArrayD::from_shape_vec(shape, chunk_vector)?
                };
                let slice_elements: Vec<SliceInfoElem> = node_key.chunk_offsets
                    [..node_key.chunk_offsets.len() - 1]
                    .iter()
                    .zip(chunk_shape[..chunk_shape.len() - 1].iter())
                    .map(|(&offset, &shape)| SliceInfoElem::Slice {
                        start: offset as isize,
                        end: Some(offset as isize + shape as isize),
                        step: 1,
                    })
                    .collect();
                let slice: SliceInfo<_, IxDyn, IxDyn> = unsafe { SliceInfo::new(slice_elements)? };
                array.slice_mut(slice).assign(&chunk_array);
            }
        }
        log::info!("Array {:#?}", &array[[100, 100, 1]]);
        Ok(array.into_dimensionality()?)
    }
}
