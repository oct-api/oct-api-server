use core::time::Duration;
use tokio::time::sleep;
use serde_json::Value;
use crate::types::*;

pub async fn start() -> Result<()> {
    let r = std::process::Command::new("./orm/manage.py")
        .args(&vec!["migrate"])
        .status()
        .expect("failed to run django migrate");
    if !r.success() {
        panic!("django migrate failed");
    }
    std::process::Command::new("./orm/manage.py")
        .args(&vec!["runserver", &config().orm_addr])
        .spawn()
        .expect("failed to start django server");
    for _ in 1..5 {
        match orm_get("/user/?username=foo", false).await {
            Ok(_) => { break; }
            _ => {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        }
    }
    Ok(())
}

fn orm_url(path: &str) -> String {
    format!("http://{}{}", config().orm_addr, path)
}

async fn orm_get(path: &str, multi: bool) -> Result<Value> {
    let resp = reqwest::get(&orm_url(path))
        .await?
        .text()
        .await?;
    let v: Vec<Value> = serde_json::from_str(&resp)?;
    if multi {
        Ok(v.into())
    } else {
        if v.len() == 1 {
            Ok(v[0].clone())
        } else if v.len() > 1 {
            bail!("Found more than 1 objects");
        } else {
            bail!("Object not found");
        }
    }
}

async fn orm_post<T: Serialize>(path: &str, data: &T) -> Result<()> {
    let client = reqwest::Client::new();
    client.post(orm_url(path))
    .json(data)
    .send()
    .await?;
    Ok(())
}

async fn orm_put<T: Serialize>(path: &str, data: &T) -> Result<()> {
    let client = reqwest::Client::new();
    client.put(orm_url(path))
    .json(data)
    .send()
    .await?;
    Ok(())
}

async fn orm_delete(path: &str) -> Result<()> {
    let client = reqwest::Client::new();
    client.delete(orm_url(path))
    .send()
    .await?;
    Ok(())
}

impl OctUser {
    pub async fn get(username: &str) -> Result<OctUser> {
        let v = orm_get(&format!("/user/?username={}", username), false).await?;
        let user: OctUser = serde_json::from_value(v)?;
        Ok(user)
    }

    pub async fn create(&self) -> Result<()> {
        orm_post("/user/", self).await?;
        Ok(())
    }

    pub async fn apps(&self) -> Result<Vec<OctApp>> {
        let v = orm_get(&format!("/app/?user__username={}", self.username), true).await?;
        let r = serde_json::from_value(v)?;
        Ok(r)
    }

    pub async fn by_token(token: &str) -> Result<OctUser> {
        let v = orm_get(&format!("/user/?token={}", token), false).await?;
        let r = serde_json::from_value(v)?;
        Ok(r)
    }
}

impl OctApp {
    pub async fn get_all() -> Result<Vec<OctApp>> {
        let v = orm_get("/app/", true).await?;
        let r = serde_json::from_value(v)?;
        Ok(r)
    }

    pub async fn orm_create(&self) -> Result<()> {
        orm_post("/app/", self).await?;
        Ok(())
    }

    pub async fn update(&self) -> Result<()> {
        orm_put(&format!("/app/{}/", self.id.unwrap()), self).await?;
        Ok(())
    }

    pub async fn delete(id: i64) -> Result<()> {
        orm_delete(&format!("/app/{}/", id)).await?;
        Ok(())
    }

    pub async fn by_handle(handle: &str) -> Result<OctApp> {
        let v = orm_get(&format!("/app/?handle={}", handle), false).await?;
        let app = serde_json::from_value(v)?;
        Ok(app)
    }

    pub async fn by_name(username: &str, name: &str) -> Result<OctApp> {
        let v = orm_get(&format!("/app/?name={}&user__username={}", name, username), false).await?;
        let app = serde_json::from_value(v)?;
        Ok(app)
    }

    pub async fn event(&self, msg: &str) -> Result<()> {
        let ae = AppEvent {
            app: self.id.unwrap(),
            content: msg.to_string(),
            datetime: None,
        };
        orm_post("/event/", &ae).await?;
        Ok(())
    }

    pub async fn get_events(&self) -> Result<Vec<AppEvent>> {
        let v = orm_get(&format!("/event/?ordering=-datetime&app__id={}", self.id.unwrap_or(-1)), true).await?;
        let ret = serde_json::from_value(v)?;
        Ok(ret)
    }
}
