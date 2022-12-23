use crate::types::*;
use crate::db::DB;
use crate::stor::*;

const DB_FILENAME: &str = "db.sqlite";
const APPS_DIR: &str = "apps";
const APP_YAML_MAX_SIZE: usize = 64 << 10;

pub fn apps_dir() -> Entry {
    Entry::new(APPS_DIR)
}

impl OctApp {
    pub async fn create(uid: i64, name: &String) -> Result<OctApp> {
        let handle = gen_random_string(5);
        apps_dir().child(&handle).create_dirs()?;
        let app = OctApp {
            id: None,
            user: Some(uid),
            name: name.to_string(),
            handle,
            git_ref: None,
            git_repo: None,
            admin_token: gen_random_string(20),
        };
        app.orm_create().await?;
        Ok(app)
    }

    pub async fn get_def(&self) -> Option<AppDef> {
        let def = match self.repo().child("app.yml").read().await {
            Ok(x) => x,
            _ => { return None; }
        };
        match AppDef::from_yaml(&def) {
            Ok(x) => Some(x),
            Err(e) => {
                println!("failed to load app yaml: {}", e);
                None
            }
        }
    }

    pub fn dir(&self) -> Entry {
        apps_dir().child(&self.handle)
    }

    pub fn repo(&self) -> Entry {
        self.dir().child("repo")
    }

    pub fn db(&self) -> Result<DB> {
        let dbf = self.dir().child(DB_FILENAME);
        DB::new(&dbf.fullpath())
    }

    pub fn running(&self) -> bool {
        self.repo().exists()
    }

    pub fn status(&self) -> &'static str {
        if self.running() {
            "RUNNING"
        } else {
            "PENDING"
        }
    }
}

pub async fn sync_models(db: &DB, old: &Option<AppDef>, new: &AppDef) -> Result<()> {
    let tables = db.tables().await?;
    let user_model = ModelDef::make_user_model();
    let mut models = vec![&user_model];
    let new_models = new.models.iter();
    models.extend(new_models.as_ref());
    for model in &models {
        let model_name = &model.name;
        if tables.contains(&model.name) {
            if model_name.starts_with("__oct") {
                /* Cannot migrate internal models yet. */
                continue;
            }
            let oldmodel = old.as_ref().unwrap().get_model(model_name).unwrap();
            model.alter_table(&db, oldmodel).await?;
        } else {
            model.create_table(&db).await?;
        }
    }
    Ok(())
}

pub async fn get_repo_app_def(repo: &Entry) -> Result<AppDef> {
    let entry = repo.child("app.yml");
    if entry.size().await? as usize > APP_YAML_MAX_SIZE {
        bail!("app.yml too large");
    }
    let s = entry.read().await?;
    let r = AppDef::from_yaml(&s)?;
    Ok(r)
}
