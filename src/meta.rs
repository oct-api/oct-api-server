use std::fs;
use std::sync::Arc;
use std::process::Command;
use serde::{Serialize, Deserialize};
use hyper::Method;
use crate::types::*;
use crate::http::*;
use crate::apps::*;
use crate::auth::authenticate;

#[derive(Debug, Serialize)]
struct AppGet {
    info: OctApp,
    base_uri: String,
    status: String,
    storage_limit_mb: u32,
    storage_usage_mb: u32,
    events: Vec<AppEvent>,
}

async fn handle_app_get(_ctx: Arc<Context>, user: OctUser, _req: Request) -> Result<Response> {
    let mut ret: Vec<AppGet> = Vec::new();

    for app in user.apps().await? {
        let base_uri = format!("/a/{}", app.handle);
        let used_mb = app.dir().size().await.unwrap_or(0) >> 20;
        let events = app.get_events().await?;
        ret.push(AppGet {
            status: app.status().to_string(),
            info: app,
            base_uri,
            storage_limit_mb: 1024,
            storage_usage_mb: used_mb as u32,
            events,
        });
    }

    json_response(&ret)
}

async fn handle_app_post(_ctx: Arc<Context>, user: OctUser, req: Request) -> Result<Response> {
    let data = String::from_utf8(to_bytes(req.into_body()).await?.to_vec())?;

    #[derive(Debug, Deserialize)]
    struct AppPostRequest {
        name: String,
    }
    let apr: AppPostRequest = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(_) => { return http400("Invalid request"); },
    };
    OctApp::create(user.id.unwrap(), &apr.name).await?;
    json_response(&1)
}

async fn handle_app_put(_ctx: Arc<Context>, user: OctUser, req: Request) -> Result<Response> {
    let data = String::from_utf8(to_bytes(req.into_body()).await?.to_vec())?;
    let mut newdata: OctApp = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(_) => { return http400("Invalid json"); },
    };
    let app = if let Ok(x) = OctApp::by_name(&user.username, &newdata.name).await {
        x
    } else {
        return http404("App not found");
    };
    newdata.id = app.id;
    newdata.user = app.user;
    newdata.name = app.name;
    newdata.handle = app.handle;
    newdata.update().await?;
    json_response(&1)
}

async fn handle_app_delete(_ctx: Arc<Context>, user: OctUser, req: Request) -> Result<Response> {
    let data = String::from_utf8(to_bytes(req.into_body()).await?.to_vec())?;
    #[derive(Deserialize)]
    struct AppDeleteReq {
        name: String,
    }
    let r: AppDeleteReq = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(_) => { return http400("Invalid parameters"); },
    };
    let app = if let Ok(x) = OctApp::by_name(&user.username, &r.name).await {
        x
    } else {
        return http404("app not found");
    };
    let r = OctApp::delete(app.id.unwrap()).await?;
    json_response(&r)
}

async fn handle_app_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    let user = if let Some(x) = authenticate(&req).await {
        x
    } else {
        return http401("Invalid or empty token in request");
    };
    match req.method() {
        &Method::GET => handle_app_get(ctx, user, req).await,
        &Method::POST => handle_app_post(ctx, user, req).await,
        &Method::PUT => handle_app_put(ctx, user, req).await,
        &Method::DELETE => handle_app_delete(ctx, user, req).await,
        _ => http404("method not recognized"),
    }
}

fn check_migration(_old: &Option<AppDef>, _new: &AppDef) -> Result<()> {
    // TODO: validate model change between cur and next
    Ok(())
}

async fn sync_app(app: &OctApp) -> Result<()> {
    let appd = app.dir();
    let next = appd.child("sync-wip");
    let repod = app.repo();
    fs::remove_dir_all(&next.fullpath());
    let repo = if let Some(x) = &app.git_repo {
        x
    } else {
        bail!("Git repo is not set for app");
    };
    let opts = if let Some(r) = &app.git_ref {
        if r != "" {
            format!("-b '{}'", r)
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };
    app.event(&format!("Cloning {}...", repo)).await;
    let cmd = format!("git clone --depth 1 {} '{}' {}",
                      opts,
                      repo,
                      next.fullpath());
    let r = Command::new("sh")
                    .current_dir(appd.fullpath())
                    .args(&["-c", &cmd])
                    .status()?;

    if ! r.success() {
        let err = "Failed to clone repo";
        app.event(err);
        bail!(err);
    }
    app.event(&format!("Checking schema...")).await;
    let olddef = if app.running() {
        Some(get_repo_app_def(&repod).await?)
    } else {
        None
    };
    let newdef = get_repo_app_def(&next).await?;
    check_migration(&olddef, &newdef)?;
    app.event(&format!("Sync database...")).await;
    sync_models(&app.db()?, &olddef, &newdef).await?;
    app.event(&format!("Activating...")).await;
    fs::remove_dir_all(&repod.fullpath());
    fs::rename(&next.fullpath(), &repod.fullpath())?;
    app.event(&format!("Done, app is up!")).await;
    Ok(())
}

async fn handle_sync_post(_ctx: Arc<Context>, req: Request) -> Result<Response> {
    let user = if let Some(x) = authenticate(&req).await {
        x
    } else {
        return http401("Invalid or empty token in request");
    };
    let data = String::from_utf8(to_bytes(req.into_body()).await?.to_vec())?;
    #[derive(Deserialize)]
    struct Req { name: String }
    let req: Req = match serde_json::from_str(&data) {
        Ok(x) => x,
        Err(_) => { return http400("Invalid request"); },
    };
    let username = user.username;
    let appname = req.name;
    let app = if let Ok(x) = OctApp::by_name(&username, &appname).await {
        x
    } else {
        return http404("App not found");
    };
    if app.git_repo.is_some() {
        match sync_app(&app).await {
            Ok(_) => (),
            Err(e) => {
                let msg = format!("Failed to sync app: {}", e);
                app.event(&msg).await;
                return http400(&msg);
            },
        }
        json_response(&1)
    } else {
        http400("Repo has no associated git")
    }
}

async fn handle_sync_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    match req.method() {
        &Method::POST => handle_sync_post(ctx, req).await,
        _ => http404("method not recognized"),
    }
}

pub async fn handle_meta_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    let path = req.uri().path();
    if path == "/meta/app" {
        handle_app_request(ctx, req).await
    } else if path == "/meta/sync" {
        handle_sync_request(ctx, req).await
    } else {
        http404("")
    }
}
