use crate::opaque::models::{
    RegisterFinishRequest, RegisterStartRequest, RegisterStartResponse,
    LoginStartRequest, LoginStartResponse, LoginFinishRequest
};
use serde::de::DeserializeOwned;
use serde::Serialize;

const SERVER_URL: &str = "http://localhost:3000";

pub fn opaque_register(payload: RegisterStartRequest<'_>) -> Result<String, reqwest::Error> {
    let url = format!("{SERVER_URL}/register/start");
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).json(&payload).send()?;
    let response_body: RegisterStartResponse = response.json()?;
    Ok(response_body.start_register_response)
}

pub fn opaque_register_finish(
    payload: RegisterFinishRequest<'_>,
) -> Result<String, reqwest::Error> {
    let url = format!("{SERVER_URL}/register/finish");
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).json(&payload).send()?;
    response.text()
}

pub fn opaque_login(payload: LoginStartRequest<'_>) -> Result<String, reqwest::Error> {
    let url = format!("{SERVER_URL}/login/start");
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).json(&payload).send()?;
    let response_body: LoginStartResponse = response.json()?;
    Ok(response_body.start_login_response)
}

pub fn opaque_login_finish(
    payload: LoginFinishRequest<'_>,
) -> Result<String, reqwest::Error> {
    let url = format!("{SERVER_URL}/login/finish");
    let client = reqwest::blocking::Client::new();
    let response = client.post(&url).json(&payload).send()?;
    response.text()
}

// Peut etre remplacer en api_port général comme ça :
//pub fn api_post<Req, Res>(path: &str, payload: &Req) -> Result<Res, reqwest::Error>
//where
//    Req: Serialize + ?Sized,
//    Res: DeserializeOwned,
//{
//    let client = reqwest::blocking::Client::new();
//    let url = format!("{SERVER_URL}{path}");
//    let response = client.post(url).json(payload).send()?;
//    Ok(response.json::<Res>()?)
//}