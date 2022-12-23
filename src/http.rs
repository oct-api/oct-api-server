use std::fs;
use std::sync::Arc;
use std::net::SocketAddr;
use std::convert::Infallible;
use std::collections::HashMap;
use url::form_urlencoded;
use hyper::service::{make_service_fn, service_fn};
use hyper::Response as HyperResponse;
use hyper::Body;
pub use hyper::body::to_bytes;
use tokio_util::codec::{BytesCodec, FramedRead};
use tokio::fs::File;
use crate::types::*;
use crate::auth::handle_auth_request;
use crate::meta::handle_meta_request;
use crate::api::handle_api_request;
use crate::alert::alert;

pub fn http500(msg: &str) -> Result<Response> {
    Ok(HyperResponse::builder().status(500).body(msg.to_string().into())?)
}

pub fn http404(msg: &str) -> Result<Response> {
    Ok(HyperResponse::builder().status(404).body(msg.to_string().into())?)
}

pub fn http401(msg: &str) -> Result<Response> {
    Ok(HyperResponse::builder().status(401).body(msg.to_string().into())?)
}

pub fn http400(msg: &str) -> Result<Response> {
    Ok(HyperResponse::builder().status(400).body(msg.to_string().into())?)
}

pub fn http302(url: &str) -> Result<Response> {
    Ok(HyperResponse::builder()
        .header("Location", url)
        .status(302)
        .body(url.to_string().into())?)
}

pub fn get_query(req: &Request) -> HashMap::<String, String> {
    let params: HashMap<String, String> = req.uri().query().map(|v| {
        form_urlencoded::parse(v.as_bytes())
            .into_owned()
            .collect()
    })
    .unwrap_or_else(HashMap::new);
    params
}

pub fn json_response<T: Serialize + ?Sized>(x: &T) -> Result<Response> {
    let resp = serde_json::to_string(x)? + "\n";
    let r = hyper::Response::builder()
        .status(200)
        .header("Content-type", "application/json")
        .body(resp.into())?;
    Ok(r)
}

pub async fn simple_response(content: String) -> Result<Response> {
    Ok(HyperResponse::new(content.into()))
}

fn guess_content_type(filename: &str) -> &'static str {
    /*
     * Gateways or load balancers such as Traefik may insert a poorer Content-type if we don't, and
     * web browsers may reject media files or scripts due to that. So, cover the most common file
     * extensions here.
     */
    if filename.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if filename.ends_with(".js") {
        "text/javascript; charset=utf-8"
    } else if filename.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".gif") {
        "image/gif"
    } else {
        "text/plain"
    }
}

pub async fn file_response(filename: &str) -> Result<Response> {
    match File::open(filename).await {
        Ok(file) => {
            let stream = FramedRead::new(file, BytesCodec::new());
            let body = Body::wrap_stream(stream);
            let resp = hyper::Response::builder()
            .status(200)
            .header("Content-type", guess_content_type(filename))
            .body(body)?;
            Ok(resp)
        },
        Err(e) => {
            println!("err: {}", e);
            http404("Not found")
        }
    }
}

async fn handle_status_request(_: Arc<Context>, req: Request) -> Result<Response> {
    #[derive(Serialize)]
    struct StatusResponse {
        status: &'static str,
    }
    json_response(&StatusResponse {
        status: "RUNNING",
    })
}

async fn handle_feedback_request(_: Arc<Context>, req: Request) -> Result<Response> {
    let data = String::from_utf8(to_bytes(req.into_body()).await?.to_vec())?;
    #[derive(Deserialize)]
    struct FeedbackReq {
        email: String,
        comments: String,
    }
    let r: FeedbackReq = serde_json::from_str(&data)?;

    let msg = format!("User feedback from '{}': {}", r.email, r.comments);
    alert(&msg);
    json_response(&true)
}

async fn handle_request(ctx: Arc<Context>, req: Request) -> Result<Response> {
    let path = String::from(req.uri().path());
    let r = if path.starts_with("/a/") {
        Some(handle_api_request(ctx, req).await)
    } else if path.starts_with("/auth/") {
        Some(handle_auth_request(ctx, req).await)
    } else if path.starts_with("/meta/") {
        Some(handle_meta_request(ctx, req).await)
    } else if path.starts_with("/feedback/") {
        Some(handle_feedback_request(ctx, req).await)
    } else if path == "/status" {
        Some(handle_status_request(ctx, req).await)
    } else {
        None
    };
    match r {
        Some(Ok(x)) => {
            return Ok(x);
        },
        Some(Err(e)) => {
            println!("{:?}", e);
            return http500("internal error");
        },
        None => (),
    }
    if path == "/doc" {
        return http302("/doc/");
    }
    let fname = if path.starts_with("/doc/") {
        let sp = &path[5..];
        if sp.ends_with("/") || sp == "" {
            format!("./doc/site/{}{}", sp, "index.html")
        } else {
            format!("./doc/site/{}", sp)
        }
    } else {
        let tryfile = "./ui/dist".to_string() + &path;
        let default = "./ui/dist/index.html".to_string();
        match fs::metadata(&tryfile) {
            Ok(x) =>
                if x.is_file() {
                    tryfile
                } else {
                    default
                },
            _ => default
        }
    };
    file_response(&fname).await
}

pub async fn run_server(ctx: Arc<Context>) -> Result<()> {
    let addr: SocketAddr = config().server_addr.parse().unwrap();
    let ctx = ctx.clone();
    let make_svc = make_service_fn(move |_conn| {
        let ctx = ctx.clone();
        let service = service_fn(move |req| {
            handle_request(ctx.clone(), req)
        });
        async move { Ok::<_, Infallible>(service) }
    });

    let server = hyper::Server::bind(&addr).serve(make_svc);

    Ok(server.await?)
}

pub fn get_auth_token(req: &Request) -> Option<&str> {
    let token = if let Some(x) = req.headers().get("Authorization") {
        if let Ok(x) = x.to_str() {
            x
        } else {
            return None;
        }
    } else {
        return None;
    };
    let fs: Vec<&str> = token.split(' ').collect();
    if fs.len() != 2 {
        return None;
    }
    Some(fs[1])
}
