pub trait NetworkSerialize<'a>: Sized {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &'a [u8]) -> Result<Self, &'static str>;
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PacketType {
    Payload = 0,
    EndPayload = 1,
}

impl PacketType {
    fn from_u8(val: u8) -> Result<Self, &'static str> {
        match val {
            0 => Ok(PacketType::Payload),
            1 => Ok(PacketType::EndPayload),
            _ => Err("Invalid PacketType byte"),
        }
    }
}

#[derive(Debug)]
pub struct Packet<T> {
    pub packet_type: PacketType,
    pub data: T,
}

impl<T> Packet<T> {
    pub fn new(packet_type: PacketType, data: T) -> Self {
        Self { packet_type, data }
    }
}

impl<'a, T: NetworkSerialize<'a>> NetworkSerialize<'a> for Packet<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut buffer = vec![self.packet_type as u8];
        buffer.extend(self.data.to_bytes());
        buffer
    }

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, &'static str> {
        if bytes.is_empty() {
            return Err("Packet too short");
        }

        Ok(Self {
            packet_type: PacketType::from_u8(bytes[0])?,
            data: T::from_bytes(&bytes[1..])?,
        })
    }
}

impl<'a> NetworkSerialize<'a> for Vec<u8> {
    fn to_bytes(&self) -> Vec<u8> {
        self.clone()
    }

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, &'static str> {
        Ok(bytes.to_vec())
    }
}

impl<'a> NetworkSerialize<'a> for &'a [u8] {
    fn to_bytes(&self) -> Vec<u8> {
        self.to_vec()
    }

    fn from_bytes(bytes: &'a [u8]) -> Result<Self, &'static str> {
        Ok(bytes)
    }
}
