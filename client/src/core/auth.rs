use secrecy::{ExposeSecret, SecretSlice};

use crate::{auth, mls, storage};
use crate::auth::session::Session;
use crate::error::AppError;

// Register a new user with OPAQUE
pub fn signup(username: &str, password: &SecretSlice<u8>) -> Result<(), AppError> {
    auth::opaque::register(username, password.expose_secret())?;
    Ok(())
}

// Login and return a fully initialized session
pub fn login(username: &str, password: &SecretSlice<u8>) -> Result<Session, AppError> {
    // Retrieve login results and create new session with it if successful
    let mut session = Session::new(auth::opaque::login(username, password.expose_secret())?);

    // Init local storage and get final token (session token)
    let (device_id, db_key, is_new_device) =
        storage::database::init_device_storage(session.user_id(), session.export_key())?;

    // Save device and storage info in the session
    session.set_device_id(&device_id);
    session.set_db_key(db_key);

    // Open DB and prepare OpenMLS provider
    session.set_provider()?;

    // Init OpenMLS
    // TODO remove unwrap when possible
    let _ = mls::identity::init_openmls(
        session.db_key().unwrap(),
        session.user_id(),
        session.device_id().unwrap(),
        session.provider().unwrap(),
        is_new_device,
    );

    Ok(session)
}
