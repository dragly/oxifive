use num_enum::TryFromPrimitive;

#[repr(u8)]
#[derive(Clone, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum MessageType {
    Nil = 0x0000,
    Dataspace = 0x0001,
    LinkInfo = 0x0002,
    Datatype = 0x0003,
    FillvalueOld = 0x0004,
    Fillvalue = 0x0005,
    Link = 0x0006,
    ExternalDataFiles = 0x0007,
    DataStorage = 0x0008,
    Bogus = 0x0009,
    GroupInfo = 0x000A,
    DataStorageFilterPipeline = 0x000B,
    Attribute = 0x000C,
    ObjectComment = 0x000D,
    ObjectModificationTimeOld = 0x000E,
    SharedMsgTable = 0x000F,
    ObjectContinuation = 0x0010,
    SymbolTable = 0x0011,
    ObjectModificationTime = 0x0012,
    BtreeKValue = 0x0013,
    DriverInfo = 0x0014,
    AttributeInfo = 0x0015,
    ObjectReferenceCount = 0x0016,
    FileSpaceInfo = 0x0018,
}

#[derive(Clone, Debug)]
pub struct MessageHeaderV1 {
    pub message_type: MessageType,
    pub size: u16,
    pub flags: u8,
    pub reserved: [u8; 3],
}

#[derive(Clone, Debug)]
pub struct MessageHeaderV2 {
    pub message_type: MessageType,
    pub size: u16,
    pub flags: u8,
}
