use bytes::{Buf, BufMut};
use displaydoc::Display as DisplayDoc;
use prost::{DecodeError, EncodeError};
use std::fmt::Display;
use thiserror::Error;

/// Prost Message Wrapper allowing serialization/deserialization
// pub(crate) struct ProstMessageSerdelizer<T: prost::Message>(pub(crate) T);

// impl<T> ProstMessageSerdelizer<T>
// where
//     T: prost::Message + std::default::Default,
// {
//     /// Method to serialize
//     pub(crate) fn serialize(&self) -> anyhow::Result<Vec<u8>> {
//         let mut buf = Vec::new();
//         self.0.encode(&mut buf).map_err(|e| anyhow::anyhow!("prost encoding error {:?}", e))?;
//         Ok(buf)
//     }

//     /// Method to deserialize
//     #[allow(dead_code)]
//     pub(crate) fn deserialize(buf: Vec<u8>) -> anyhow::Result<T> {
//         T::decode(Bytes::from(buf)).map_err(|e| anyhow::anyhow!("prost decoding error {:?}", e))
//     }
// }

/// Serialization/Deserialization error types.
#[derive(Debug, DisplayDoc, Error)]
pub enum Error {
    /// error encoding message into buffer: {0}
    Encode(#[from] EncodeError),
    /// error decoding buffer into message: {0}
    Decode(#[from] DecodeError),
    /// error converting message type into domain type: {0}
    TryFromProtobuf(String),
}

impl Error {
    pub fn try_from<Raw, T, E>(e: E) -> Error
    where
        E: Display,
        T: TryFrom<Raw, Error = E>,
    {
        Error::TryFromProtobuf(format!("{e}"))
    }
}

pub trait Protobuf<T: prost::Message + From<Self> + Default>
where
    Self: Sized + Clone + TryFrom<T>,
    <Self as TryFrom<T>>::Error: Display,
{
    /// Encode into a buffer in Protobuf format.
    ///
    /// Uses [`prost::Message::encode`] after converting into its counterpart
    /// Protobuf data structure.
    ///
    /// [`prost::Message::encode`]: https://docs.rs/prost/*/prost/trait.Message.html#method.encode
    fn encode<B: BufMut>(self, buf: &mut B) -> Result<(), Error> {
        T::from(self).encode(buf).map_err(Error::Encode)
    }

    /// Encode with a length-delimiter to a buffer in Protobuf format.
    ///
    /// An error will be returned if the buffer does not have sufficient capacity.
    ///
    /// Uses [`prost::Message::encode_length_delimited`] after converting into
    /// its counterpart Protobuf data structure.
    ///
    /// [`prost::Message::encode_length_delimited`]: https://docs.rs/prost/*/prost/trait.Message.html#method.encode_length_delimited
    fn encode_length_delimited<B: BufMut>(self, buf: &mut B) -> Result<(), Error> {
        T::from(self).encode_length_delimited(buf).map_err(|e| Error::Encode(e))
    }

    /// Constructor that attempts to decode an instance from a buffer.
    ///
    /// The entire buffer will be consumed.
    ///
    /// Similar to [`prost::Message::decode`] but with additional validation
    /// prior to constructing the destination type.
    ///
    /// [`prost::Message::decode`]: https://docs.rs/prost/*/prost/trait.Message.html#method.decode
    fn decode<B: Buf>(buf: B) -> Result<Self, Error> {
        let raw = T::decode(buf).map_err(Error::Decode)?;

        Self::try_from(raw).map_err(Error::try_from::<T, Self, _>)
    }

    /// Constructor that attempts to decode a length-delimited instance from
    /// the buffer.
    ///
    /// The entire buffer will be consumed.
    ///
    /// Similar to [`prost::Message::decode_length_delimited`] but with
    /// additional validation prior to constructing the destination type.
    ///
    /// [`prost::Message::decode_length_delimited`]: https://docs.rs/prost/*/prost/trait.Message.html#method.decode_length_delimited
    fn decode_length_delimited<B: Buf>(buf: B) -> Result<Self, Error> {
        let raw = T::decode_length_delimited(buf).map_err(Error::Decode)?;

        Self::try_from(raw).map_err(Error::try_from::<T, Self, _>)
    }

    /// Returns the encoded length of the message without a length delimiter.
    ///
    /// Uses [`prost::Message::encoded_len`] after converting to its
    /// counterpart Protobuf data structure.
    ///
    /// [`prost::Message::encoded_len`]: https://docs.rs/prost/*/prost/trait.Message.html#method.encoded_len
    fn encoded_len(self) -> usize {
        T::from(self).encoded_len()
    }

    /// Encodes into a Protobuf-encoded `Vec<u8>`.
    fn encode_vec(self) -> Vec<u8> {
        T::from(self).encode_to_vec()
    }

    /// Constructor that attempts to decode a Protobuf-encoded instance from a
    /// `Vec<u8>` (or equivalent).
    fn decode_vec(v: &[u8]) -> Result<Self, Error> {
        Self::decode(v)
    }

    /// Encode with a length-delimiter to a `Vec<u8>` Protobuf-encoded message.
    fn encode_length_delimited_vec(self) -> Vec<u8> {
        T::from(self).encode_length_delimited_to_vec()
    }

    /// Constructor that attempts to decode a Protobuf-encoded instance with a
    /// length-delimiter from a `Vec<u8>` or equivalent.
    fn decode_length_delimited_vec(v: &[u8]) -> Result<Self, Error> {
        Self::decode_length_delimited(v)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_prost_simple() {
        /// This is the protobuf type for a e.g. Block
        use messaging::protobuftypes::MyDemoType;
        use prost::Message;

        /// This is analogous to a domain type in a fuel-core. e.g. Block
        #[derive(Clone, Debug)]
        pub struct MyDomainType {
            age: i32,
            name: String,
            is_active: bool,
        }

        impl Protobuf<MyDemoType> for MyDomainType {}

        impl From<MyDomainType> for MyDemoType {
            fn from(domain: MyDomainType) -> Self {
                MyDemoType { age: domain.age, name: domain.name, is_active: domain.is_active }
            }
        }

        impl TryFrom<MyDemoType> for MyDomainType {
            type Error = Error;

            fn try_from(proto: MyDemoType) -> Result<Self, Self::Error> {
                Ok(Self { age: proto.age, name: proto.name, is_active: proto.is_active })
            }
        }

        let my_demo_type = MyDemoType { age: 9, name: "Albert".to_string(), is_active: false };

        let mut valid_raw_bytes: Vec<u8> = Vec::new();
        my_demo_type.encode(&mut valid_raw_bytes).unwrap();
        assert!(!valid_raw_bytes.is_empty());

        let my_domain_type = MyDomainType::decode(valid_raw_bytes.clone().as_ref()).unwrap();
        assert!(my_domain_type.age == 9);
        assert!(my_domain_type.is_active == false);
        assert!(my_domain_type.name == "Albert".to_string());
    }

    #[test]
    fn test_prost_complex() {
        use messaging::protobuftypes::{request::ReqType, PubKeyRequest, PubKeyResponse, Request};
        use prost::Message;

        // =================== DOMAIN ====================
        #[derive(Clone, Debug)]
        pub struct PubKeyRequestDomain {
            id: String,
            data: Vec<u8>,
        }
        impl Protobuf<PubKeyRequest> for PubKeyRequestDomain {}

        impl From<PubKeyRequestDomain> for PubKeyRequest {
            fn from(domain: PubKeyRequestDomain) -> Self {
                PubKeyRequest { id: domain.id, data: domain.data }
            }
        }

        impl TryFrom<PubKeyRequest> for PubKeyRequestDomain {
            type Error = Error;

            fn try_from(proto: PubKeyRequest) -> Result<Self, Self::Error> {
                Ok(Self { data: proto.data, id: proto.id })
            }
        }

        // =================== DOMAIN ====================
        #[derive(Clone, Debug)]
        pub struct PubKeyResponseDomain {
            id: String,
            data: Vec<u8>,
        }
        impl Protobuf<PubKeyResponse> for PubKeyResponseDomain {}

        impl From<PubKeyResponseDomain> for PubKeyResponse {
            fn from(domain: PubKeyResponseDomain) -> Self {
                PubKeyResponse { id: domain.id, data: domain.data }
            }
        }

        impl TryFrom<PubKeyResponse> for PubKeyResponseDomain {
            type Error = Error;

            fn try_from(proto: PubKeyResponse) -> Result<Self, Self::Error> {
                Ok(Self { data: proto.data, id: proto.id })
            }
        }

        // =================== DOMAIN ====================
        #[derive(Clone, Debug)]
        pub enum ReqTypeDomain {
            PubKeyRequest(PubKeyRequestDomain),
            PubKeyResponse(PubKeyResponseDomain),
        }

        // impl Protobuf<ReqType> for ReqTypeDomain {}

        impl From<ReqTypeDomain> for ReqType {
            fn from(domain: ReqTypeDomain) -> Self {
                match domain {
                    ReqTypeDomain::PubKeyRequest(domain) => ReqType::PubKeyRequest(domain.into()),
                    ReqTypeDomain::PubKeyResponse(domain) => ReqType::PubKeyResponse(domain.into()),
                }
            }
        }

        impl TryFrom<ReqType> for ReqTypeDomain {
            type Error = Error;

            fn try_from(proto: ReqType) -> Result<Self, Self::Error> {
                match proto {
                    ReqType::PubKeyRequest(proto) => {
                        Ok(ReqTypeDomain::PubKeyRequest(proto.try_into()?))
                    }
                    ReqType::PubKeyResponse(proto) => {
                        Ok(ReqTypeDomain::PubKeyResponse(proto.try_into()?))
                    }
                }
            }
        }

        // =================== TEST ====================
        // create protobuf types
        let request = Request {
            req_type: Some(ReqType::PubKeyRequest(PubKeyRequest {
                id: "1".to_string(),
                data: vec![1, 2, 3, 4],
            })),
        };

        let mut valid_raw_bytes: Vec<u8> = Vec::new();
        request.encode(&mut valid_raw_bytes).unwrap();
        assert!(!valid_raw_bytes.is_empty());

        // decode to domain type
        let my_domain_type = PubKeyRequestDomain::decode(valid_raw_bytes.clone().as_ref()).unwrap();
    }
}
