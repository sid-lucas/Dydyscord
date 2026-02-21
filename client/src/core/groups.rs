use crate::{mls, storage, transport};
use crate::auth::session::Session;
use crate::error::AppError;

// Add a friend placeholder until server flow exists
pub fn add_friend(_username: &str) -> Result<(), AppError> {
    Ok(())
}

// Create a new group and invite a user
pub fn create_group(
    session: &Session,
    group_name: &str,
    invite_username: &str,
) -> Result<(), AppError> {
    // TODO remove unwrap when possible
    mls::identity::init_group(
        session.db_key().unwrap(),
        session.user_id(),
        session.device_id().unwrap(),
        session.provider().unwrap(),
        invite_username,
        group_name,
    )?;
    Ok(())
}

// Load groups from local storage for the current user
pub fn browse_groups(
    session: &Session,
) -> Result<Vec<(openmls::prelude::GroupId, String)>, AppError> {
    let groups =
        storage::database::retrieve_groups(session.db_key().unwrap(), session.user_id()).unwrap();
    Ok(groups)
}

// Fetch welcome messages for the current session
pub fn fetch_welcome(session: &Session) -> Result<(), AppError> {
    // TODO remove unwrap when possible
    mls::identity::fetch_welcome(
        session.db_key().unwrap(),
        session.user_id(),
        session.provider().unwrap(),
    )?;
    Ok(())
}

// Test the current session token against the server
pub fn test_session() -> Result<(), AppError> {
    transport::api::test_session()?;
    Ok(())
}
