use openmls_rust_crypto::RustCrypto;
use openmls_sqlite_storage::{Connection, SqliteStorageProvider};
use openmls_traits::OpenMlsProvider;
use secrecy::SecretSlice;

use crate::{
    error::AppError,
    mls::error::MlsError,
    storage::{codec::CBORCodec, database}
};

pub struct MyProvider {
    crypto: RustCrypto,
    rand: RustCrypto,
    storage: SqliteStorageProvider<CBORCodec, Connection>,
}

impl OpenMlsProvider for MyProvider {
    type CryptoProvider = RustCrypto;
    type RandProvider = RustCrypto;
    type StorageProvider = SqliteStorageProvider<CBORCodec, Connection>;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }
    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }
    fn rand(&self) -> &Self::RandProvider {
        &self.rand
    }
}

pub fn prepare_provider(db_key: &SecretSlice<u8>, user_id: &str) -> Result<MyProvider, AppError> {
    let conn = database::open_sqlcipher(db_key, user_id)?;

    let mut storage = SqliteStorageProvider::<CBORCodec, _>::new(conn);

    storage.run_migrations().map_err(|_| MlsError::Migration)?;

    Ok(MyProvider {
        crypto: RustCrypto::default(),
        rand: RustCrypto::default(),
        storage,
    })
}
