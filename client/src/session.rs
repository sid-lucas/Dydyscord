use crate::error::ClientError;
use crate::opaque;

pub struct Session {
    pub user_id: i32,
    pub device_id: String,
    pub export_key: Vec<u8>,
    pub session_key: Vec<u8>,
    pub db_key: Vec<u8>,
}

pub fn login() -> Result<(), ClientError> {
    let login_result = opaque::auth::login();
    let (user_id, export_key, session_key) = match login_result {
        Ok(login_result) => (
            login_result.id,
            login_result.export_key,
            login_result.session_key,
        ),
        Err(e) => return Err(e.into()),
    };

    let my_session = Session {
        user_id,
        device_id: String::from("temp"), // TODO : temporaire pour compilation
        export_key,
        session_key,
        db_key: Vec::new(), // TODO : temporaire pour compilation
    };

    Ok(())
}
