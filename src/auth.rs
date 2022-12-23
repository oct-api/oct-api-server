use std::env;
use std::fs;
use std::sync::Arc;
use reqwest;
use serde_json::{Value};
use crate::types::*;
use crate::http::*;
use crate::alert::*;

const GITHUB_CLIENT_ID_FILE: &str = "secrets/github-oauth/client_id";
const GITHUB_CLIENT_SECRET_FILE: &str = "secrets/github-oauth/client_secret";
const GITHUB_CLIENT_REDIRECT_URI_FILE: &str = "secrets/github-oauth/redirect_uri_base";

#[derive(Debug, Serialize)]
struct GithubAuthConfig {
    client_id: String,
    client_secret: String,
    redirect_uri_base: String,
}

fn read_secret_file(p: &str, def: &str) -> String {
    match fs::read_to_string(p) {
        Ok(s) => s.trim_end().to_string(),
        _ => def.to_string(),
    }
}

fn github_auth_config() -> GithubAuthConfig {
    let client_id = read_secret_file(GITHUB_CLIENT_ID_FILE,
        "");
    let client_secret = read_secret_file(GITHUB_CLIENT_SECRET_FILE,
        "");
    let redirect_uri_base = match env::var("OCT_DOMAIN_NAME") {
        Ok(s) => format!("https://{}", s),
        _ => "http://localhost:8888".to_string(),
    };
    GithubAuthConfig {
        client_id,
        client_secret,
        redirect_uri_base,
    }
}

async fn handle_github_callback(_ctx: Arc<Context>, req: Request) -> Result<Response> {
    let query = get_query(&req);
    let code = if let Some(x) = query.get("code") {
        x
    } else {
        return http400("Cannot find code paramter");
    };
    let client = reqwest::Client::new();
    let cfg = github_auth_config();
    let params = [("client_id", &cfg.client_id),
                  ("client_secret", &cfg.client_secret),
                  ("code", code)];
    let res: Value = client.post("https://github.com/login/oauth/access_token")
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await?
        .json()
        .await?;
    let token = if let Some(x) = res.get("access_token") {
        x.as_str().unwrap_or("")
    } else {
        return http400("cannot get access token");
    };

    let res: Value = client.get("https://api.github.com/user")
        .header("Authorization", &format!("token {}", token))
        .header("Accept", "application/json")
        .header("User-agent", "reqwest")
        .send()
        .await?
        .json()
        .await?;
    let gh_account = if let Some(x) = res.get("login") {
        x.as_str().unwrap_or("")
    } else {
        return http400("cannot get username from github api");
    };
    let email = if let Some(x) = res.get("email") {
        x.as_str().unwrap_or("")
    } else {
        return http400("cannot get username from github api");
    };
    let username = format!("github.{}", gh_account);
    alert(&format!("User login with GitHub: {}", gh_account));
    let token = match OctUser::get(&username).await {
        Ok(x) => x.token,
        _ => {
            let user = OctUser::new(&username, gh_account, email);
            user.create().await?;
            user.token
        },
    };

    http302(&format!("{}/login/confirm/{}", cfg.redirect_uri_base, token))
}

pub async fn authenticate(req: &Request) -> Option<OctUser> {
    if let Some(x) = get_auth_token(req) {
        if let Ok(user) = OctUser::by_token(x).await {
            return Some(user);
        }
    }
    None
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthInfoResponse {
    username: String,
    display_name: String,
}

async fn handle_info(_ctx: Arc<Context>, req: Request) -> Result<Response> {
    let user = if let Some(x) = authenticate(&req).await {
        x
    } else {
        return http401("Invalid or empty token in request");
    };
    json_response(&AuthInfoResponse {
        username: user.username,
        display_name: user.display_name,
    })
}

pub async fn handle_auth_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    let path = req.uri().path();
    let cfg = github_auth_config();
    let r = if path == "/auth/github/config" {
        json_response(&cfg)
    } else if path == "/auth/info" {
        handle_info(ctx, req).await
    } else if path == "/auth/github/callback" {
        handle_github_callback(ctx, req).await
    } else {
        http404("")
    };
    if let Err(e) = r {
        return http400(&e.to_string());
    }
    r
}
