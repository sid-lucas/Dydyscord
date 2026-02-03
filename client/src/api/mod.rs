use crate::opaque::models::{
    LoginFinishRequest, LoginStartRequest, LoginStartResponse, RegisterFinishRequest,
    RegisterStartRequest, RegisterStartResponse,
};

pub enum ApiError {
    Http {
        status: reqwest::StatusCode,
        message: String,
    },
    Reqwest(reqwest::Error),
}

const SERVER_URL: &str = "http://localhost:3000";

pub fn opaque_register(
    payload: RegisterStartRequest<'_>,
) -> Result<RegisterStartResponse, ApiError> {
    let url = format!("{SERVER_URL}/register/start");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| ApiError::Reqwest(e))?;

    let status = response.status();

    if status.is_success() {
        Ok(response.json().map_err(|e| ApiError::Reqwest(e))?)
    } else {
        let error: String = response.json().map_err(|e| ApiError::Reqwest(e))?;
        Err(ApiError::Http {
            status,
            message: error,
        })
    }
}

pub fn opaque_register_finish(payload: RegisterFinishRequest<'_>) -> Result<String, ApiError> {
    let url = format!("{SERVER_URL}/register/finish");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| ApiError::Reqwest(e))?;

    let status = response.status();

    if status.is_success() {
        Ok(response.text().map_err(|e| ApiError::Reqwest(e))?)
    } else {
        let error: String = response.json().map_err(|e| ApiError::Reqwest(e))?;
        Err(ApiError::Http {
            status,
            message: error,
        })
    }
}

pub fn opaque_login(payload: LoginStartRequest<'_>) -> Result<LoginStartResponse, ApiError> {
    let url = format!("{SERVER_URL}/login/start");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| ApiError::Reqwest(e))?;

    let status = response.status();

    if status.is_success() {
        Ok(response.json().map_err(|e| ApiError::Reqwest(e))?)
    } else {
        let error: String = response.json().map_err(|e| ApiError::Reqwest(e))?;
        Err(ApiError::Http {
            status,
            message: error,
        })
    }
}

pub fn opaque_login_finish(payload: LoginFinishRequest) -> Result<String, ApiError> {
    let url = format!("{SERVER_URL}/login/finish");
    let client = reqwest::blocking::Client::new();
    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .map_err(|e| ApiError::Reqwest(e))?;

    let status = response.status();

    if status.is_success() {
        Ok(response.text().map_err(|e| ApiError::Reqwest(e))?)
    } else {
        let error: String = response.json().map_err(|e| ApiError::Reqwest(e))?;
        Err(ApiError::Http {
            status,
            message: error,
        })
    }
}
