use crate::error::AppError;
use crate::storage::error::StorageError;

pub struct CBORCodec;

impl Default for CBORCodec {
    fn default() -> Self {
        Self
    }
}

impl openmls_sqlite_storage::Codec for CBORCodec {
    type Error = AppError;

    fn to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, Self::Error> {
        let mut out = Vec::new();
        ciborium::ser::into_writer(value, &mut out)
            .map_err(|_| StorageError::CborCodecSerialize)?;
        Ok(out)
    }

    fn from_slice<T: serde::de::DeserializeOwned>(slice: &[u8]) -> Result<T, Self::Error> {
        let input =
            ciborium::de::from_reader(slice).map_err(|_| StorageError::CborCodecDeserialize)?;
        Ok(input)
    }
}