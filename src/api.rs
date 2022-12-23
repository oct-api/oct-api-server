use std::sync::Arc;
use regex::Regex;
use hyper::body::to_bytes;
use serde_json::json;
use crate::types::*;
use crate::http::*;
use crate::graphql::handle_graphql;
use crate::db::DB;

async fn handle_model_get(req: Request, db: DB, model: &ModelDef, uid: Option<i64>) -> Result<Response> {
    let qm = get_query(&req);
    let id: Option<i64> = if let Some(x) = qm.get("id") {
        x.parse().ok()
    } else {
        None
    };

    let mut ret = Vec::new();
    for rec in model.select(&db, uid, id).await? {
        ret.push(rec.to_value());
    }
    json_response(&ret)
}

async fn read_body(req: Request) -> Result<String> {
    let bytes = to_bytes(req.into_body()).await?;
    Ok(String::from_utf8(bytes.to_vec())?)
}

async fn handle_model_post(req: Request, db: DB, model: &ModelDef, uid: Option<i64>) -> Result<Response> {
    let rec = Row::from_json(&read_body(req).await?)?;
    model.create(&db, &rec, uid).await?;
    json_response(&1)
}

async fn handle_model_put(req: Request, db: DB, model: &ModelDef, uid: Option<i64>) -> Result<Response> {
    let rec = Row::from_json(&read_body(req).await?)?;
    model.update(&db, &rec, uid).await?;
    json_response(&1)
}

async fn handle_model_delete(req: Request, db: DB, model: &ModelDef, uid: Option<i64>) -> Result<Response> {
    #[derive(Deserialize)]
    struct DeleteReq {
        id: Option<i64>,
        pks: Option<Vec<i64>>,
    }
    let d: DeleteReq = match serde_json::from_str(&read_body(req).await?) {
        Ok(x) => x,
        Err(_) => { return http400("Invalid paramters"); }
    };
    let mut pks = d.pks.unwrap_or(Vec::new());
    if let Some(id) = d.id {
        pks.push(id);
    }
    let r = model.delete(&db, &pks[..], uid).await?;
    json_response(&r)
}

async fn handle_model_request(req: Request,
                              app: &OctApp,
                              m: &ModelApiDesc,
                              uid: Option<i64>) -> Result<Response> {
    let db = app.db()?;
    let def = if let Some(x) = app.get_def().await {
        x
    } else {
        return http404("model not found");
    };
    let model = if let Some(m) = def.get_model(&m.model) {
        m
    } else {
        bail!("Model not found");
    };
    match req.method() {
        &hyper::Method::GET => handle_model_get(req, db, &model, uid).await,
        &hyper::Method::POST => handle_model_post(req, db, &model, uid).await,
        &hyper::Method::PUT => handle_model_put(req, db, &model, uid).await,
        &hyper::Method::DELETE => handle_model_delete(req, db, &model, uid).await,
        _ => bail!("Unsupported method"),
    }
}

async fn handle_status_request(_ctx: Arc<Context>, app: &OctApp) -> Result<Response> {
    let resp = json!({
        "status": app.status(),
    });
    json_response(&resp)
}

async fn handle_stats_request(ctx: Arc<Context>, app: &OctApp) -> Result<Response> {
    let resp = ctx.stats()
        .get_by_prefix(&format!("api.{}.", app.handle));
    json_response(&resp)
}

async fn handle_query_count_request(ctx: Arc<Context>, req: Request, app: &OctApp) -> Result<Response> {
    let pref = format!("api.{}.", app.handle);
    let qm = get_query(&req);
    let unit = match qm.get("unit") {
        Some(x) => match x.as_str() {
            "hour" => TimeSeriesUnit::Hourly,
            "day" => TimeSeriesUnit::Daily,
            _ => TimeSeriesUnit::Minutely,
        },
        _ => TimeSeriesUnit::Hourly,
    };
    let ts = ctx.stats().time_series_by_prefix(&pref, &unit);
    json_response(&ts)
}

#[derive(Debug, Serialize)]
struct ApiAuthUserGet {
    name: String,
    email: String,
}

impl ApiAuthUserGet {
    fn from(rec: &Row) -> ApiAuthUserGet {
        ApiAuthUserGet {
            name: rec.get_str("name").unwrap_or("").to_string(),
            email: rec.get_str("email").unwrap_or("").to_string(),
        }
    }
}

async fn handle_api_auth_get(_ctx: Arc<Context>, _req: Request, app: &OctApp,
                             api_path: &str) -> Result<Response> {

    if api_path == "/auth/user" {
        let db = app.db()?;
        let model = ModelDef::make_user_model();
        let recs = model.select(&db, None, None).await?;
        let res: Vec<ApiAuthUserGet> = 
            recs.iter()
            .map(|x| ApiAuthUserGet::from(x))
            .collect();
        json_response(&res)
    } else {
        http404("not found")
    }
}

async fn handle_api_auth_post(_ctx: Arc<Context>, req: Request, app: &OctApp,
                              api_path: &str) -> Result<Response> {
    if api_path == "/auth/user" {
        let db = app.db()?;
        let model = ModelDef::make_user_model();
        let rec = Row::from_json(&read_body(req).await?)?;
        model.create(&db, &rec, None).await?;
        return json_response(&1);
    } else {
        return http404("not found");
    }
}

async fn handle_api_auth_request(ctx: Arc<Context>, req: Request, app: &OctApp,
                                 api_path: &str, uid: Option<i64>) -> Result<Response> {
    if uid.unwrap_or(-1) != 0 {
        return http401("Permission denied");
    }
    match req.method() {
        &hyper::Method::GET => handle_api_auth_get(ctx, req, app, api_path).await,
        &hyper::Method::POST => handle_api_auth_post(ctx, req, app, api_path).await,
        _ => bail!("Unsupported method"),
    }
}

async fn api_authenticate(_ctx: Arc<Context>, app: &OctApp,
                          req: &Request) -> Result<Option<i64>> {
    let token = if let Some(x) = get_auth_token(req) {
        x
    } else {
        return Ok(None);
    };
    if token.contains('\'') {
        return Ok(None);
    }
    if token == app.admin_token {
        return Ok(Some(0));
    }
    let db = app.db()?;
    let cond = format!("token='{}'", token);
    let rec = match db.get(&ModelDef::make_user_model(), &cond) {
        Ok(Some(x)) => x,
        _ => { return Ok(None) },
    };
    match rec.get("id") {
        Some(RowField::Integer(x)) => Ok(Some(*x)),
        _ => Ok(None),
    }
}

async fn rule_match(_ctx: &Arc<Context>,
                    _app: &OctApp,
                    method_name: &str,
                    rule: &ApiAccessRuleDef,
                    uid: Option<i64>) -> bool {
    if let Some(x) = &rule.method {
        if x.as_str() != method_name {
            return false;
        }
    }
    match &rule.role {
        None => true,
        Some(x) => match x.as_str() {
            "anonymous" => uid.is_none(),
            "admin" => uid == Some(0),
            "user" => uid.unwrap_or(-1) > 0,
            _ => false,
        },
    }
}

async fn check_access(ctx: Arc<Context>,
                      req: &Request,
                      app: &OctApp,
                      ep: &ApiEndpoint,
                      uid: Option<i64>) -> Result<bool> {
    if Some(0) == uid {
        /* Admin can do everything */
        return Ok(true);
    }
    let method_name = req.method().as_str();
    let appdef = if let Some(x) = app.get_def().await {
        x
    } else {
        return Ok(false);
    };
    let defrules;
    let rules = if let Some(x) = ep.access() {
        x
    } else {
        if let Some(x) = &appdef.api.default_access {
            defrules = x.get_rules();
            &defrules
        } else {
            return Ok(false);
        }
    };
    for rule in rules {
        if rule_match(&ctx, app, &method_name.to_lowercase(), &rule, uid).await {
            return Ok(rule.action.allowed());
        }
    }
    Ok(false)
}

pub async fn handle_api_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let re = Regex::new(r"/a/([A-Z]+)(/.*)").unwrap();
    let cap = if let Some(x) = re.captures(&path) {
        x
    } else {
        return http404("Incomplete URI path");
    };
    let handle = cap.get(1).unwrap().as_str();
    let api_path = cap.get(2).unwrap().as_str();

    let app = if let Ok(x) = OctApp::by_handle(handle).await {
        x
    } else {
        return http404("App not found");
    };
    let uid = api_authenticate(ctx.clone(), &app, &req).await?;

    if api_path == "/__oct_status" {
        return handle_status_request(ctx.clone(), &app).await;
    } else if api_path == "/__oct_stats" {
        return handle_stats_request(ctx.clone(), &app).await;
    } else if api_path == "/__oct_query_count" {
        return handle_query_count_request(ctx.clone(), req, &app).await;
    } else if api_path.starts_with("/auth") {
        return handle_api_auth_request(ctx.clone(), req, &app, api_path, uid).await;
    }
    let appdef = if let Some(x) = app.get_def().await {
        x
    } else {
        return http404("API not found");
    };
    if let Some(ep) = appdef.api.find_endpoint(api_path) {
        if !check_access(ctx.clone(), &req, &app, &ep, uid).await? {
            return http401("Permission error");
        }
        let h = match &ep {
            ApiEndpoint::String(c) => simple_response(c.response.to_string()).await,
            ApiEndpoint::StaticFile(_) => {
                bail!("not implemented");
                //file_response(&c.localfile).await
            },
            ApiEndpoint::Model(m) => handle_model_request(req, &app, &m, uid).await,
            ApiEndpoint::GraphQL(def) => handle_graphql(req, &app, &def, uid).await,
        };
        let metric = format!("api.{}.{}.{}",
            handle, ep.name(), &method);
        ctx.stats().account(&metric);
        h
    } else {
        http404(&format!("Endpoint '{}' not found", path))
    }
}
