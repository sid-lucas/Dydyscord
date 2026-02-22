use secrecy::SecretSlice;

use crate::{
    auth::{self, session::Session},
    error::AppError,
    mls, storage,
};

pub fn perform_login(username: &str, password: &SecretSlice<u8>) -> Result<Session, AppError> {
    // Retrieve login results and create new session with it if successful
    let mut session = Session::new(auth::opaque::login(username, password)?);

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

pub fn perform_signup(username: &str, password: &SecretSlice<u8>) -> Result<(), AppError> {
    // Try to register with OPAQUE
    auth::opaque::register(&username, password)?;

    // Go back to main menu
    Ok(())
}
