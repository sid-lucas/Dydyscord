use OpaqueCipherSuite as Default;
use base64::Engine;
use common::{
    OpaqueLoginFinishRequest, OpaqueLoginStartRequest, OpaqueRegisterFinishRequest,
    OpaqueRegisterStartRequest,
};
use opaque_ke::argon2::Argon2;
use opaque_ke::{
    CipherSuite, ClientLogin, ClientLoginFinishParameters, ClientRegistration,
    ClientRegistrationFinishParameters, CredentialResponse, RegistrationResponse,
};
use rand::rngs::OsRng;
use uuid::Uuid;

use crate::auth::error::AuthError;
use crate::error::AppError;
use crate::transport::http;

struct OpaqueCipherSuite;

impl CipherSuite for OpaqueCipherSuite {
    type OprfCs = opaque_ke::Ristretto255;
    type KeyExchange = opaque_ke::TripleDh<opaque_ke::Ristretto255, sha2::Sha512>;
    type Ksf = Argon2<'static>;
}

pub struct LoginResult {
    pub username: String,
    pub id: Uuid,
    pub export_key: Vec<u8>, // TODO REVIEW with SecretSlice<u8>
    pub session_key: Vec<u8>,
}

pub fn register(username: &str, password: &str) -> Result<(), AppError> {
    let mut client_rng = OsRng;

    // Start client registration with OPAQUE
    let start = ClientRegistration::<Default>::start(&mut client_rng, &password.as_bytes())
        .map_err(|_| AuthError::OpaqueRegisterStart)?;

    // Prepare the request to send to the server
    let start_register_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // API call (send request and receive response)
    let response = http::opaque_register(OpaqueRegisterStartRequest {
        username: username.to_string(),
        request_b64: start_register_request,
    })?;
    let register_response_b64 = response.response_b64;

    // Response base64 -> bytes
    let register_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&register_response_b64)
        .map_err(|_| AuthError::OpaqueDecode)?;
    // Response deserialization
    let register_response = RegistrationResponse::<Default>::deserialize(&register_response_bytes)
        .map_err(|_| AuthError::OpaqueDeserialize)?;

    // Start finish with the server response
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password.as_bytes(),
            register_response,
            ClientRegistrationFinishParameters::default(),
        )
        .map_err(|_| AuthError::OpaqueRegisterFinish)?;

    // Prepare the request to send to the server
    let finish_register_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    // API call (send request and receive response)
    http::opaque_register_finish(OpaqueRegisterFinishRequest {
        username: username.to_string(),
        request_b64: finish_register_request,
    })?;

    Ok(())
}

pub fn login(username: &str, password: &str) -> Result<LoginResult, AppError> {
    let mut client_rng = OsRng;

    // Start client login with OPAQUE
    let start = ClientLogin::<Default>::start(&mut client_rng, &password.as_bytes())
        .map_err(|_| AuthError::OpaqueLoginStart)?;

    // Prepare the request to send to the server
    let start_login_request =
        base64::engine::general_purpose::STANDARD.encode(start.message.serialize());

    // TODO : fix timing attack

    // API call (send request and receive response)
    let response = http::opaque_login(OpaqueLoginStartRequest {
        username: username.to_string(),
        request_b64: start_login_request,
    })?;
    let (login_response_b64, id) = (response.response_b64, response.user_id);

    // Response base64 -> bytes
    let login_response_bytes = base64::engine::general_purpose::STANDARD
        .decode(&login_response_b64)
        .map_err(|_| AuthError::OpaqueDecode)?;
    // Response deserialization
    let login_response = CredentialResponse::<Default>::deserialize(&login_response_bytes)
        .map_err(|_| AuthError::OpaqueDeserialize)?;

    // Finish login with the server response
    let finish = start
        .state
        .finish(
            &mut client_rng,
            &password.as_bytes(),
            login_response,
            ClientLoginFinishParameters::default(),
        )
        .map_err(|_| AuthError::OpaqueLoginFinish)?;

    let export_key = finish.export_key.to_vec();
    let session_key = finish.session_key.to_vec();

    // Prepare the request to send to the server+
    let finish_login_request =
        base64::engine::general_purpose::STANDARD.encode(finish.message.serialize());

    // API call (send request and receive response)
    http::opaque_login_finish(OpaqueLoginFinishRequest {
        user_id: id,
        request_b64: finish_login_request,
    })?;

    Ok(LoginResult {
        username: username.to_string(),
        id,
        export_key,
        session_key,
    })
}
