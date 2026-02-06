use crate::error::ClientError;
use crate::opaque;
use crate::session;

pub fn login_flow() -> Result<(), ClientError> {
    let login_result = opaque::auth::login();
    match login_result {
        Ok(login_result) => (),
        Err(e) => return Err(e.into()),
    }

    session::init_session(login_result);

    Ok(())
}
