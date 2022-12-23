use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;
use crate::types::*;
use crate::db::{DB, DbValue};

fn timestamp() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

impl FieldDef {
    pub fn name(&self) -> &str {
        match self {
            Self::String(d) => &d.name,
            Self::Integer(d) => &d.name,
            Self::Float(d) => &d.name,
            Self::Boolean(d) => &d.name,
            Self::DateTime(d) => &d.name,
            Self::User(d) => &d.name,
            Self::Reference(d) => &d.name,
        }
    }

    pub fn default_value(&self) -> Option<RowField> {
        match self {
            Self::String(_) => None,
            Self::Integer(_) => None,
            Self::Float(_) => None,
            Self::Boolean(_) => None,
            Self::DateTime(d) =>
                match d.default_now {
                    Some(true) => Some(RowField::DateTime(timestamp())),
                    _ => None,
                },
            Self::User(_) => None,
            Self::Reference(_) => None,
        }
    }

    fn type_sql(&self) -> String {
        let typename = match self {
            FieldDef::String(_) => "VARCHAR(128)",
            FieldDef::Integer(_) => "BIGINT",
            FieldDef::Float(_) => "FLOAT",
            FieldDef::Boolean(_) => "INTEGER",
            FieldDef::DateTime(_) => "DATETIME",
            FieldDef::User(_) => "BIGINT",
            FieldDef::Reference(_) => "BIGINT",
        };
        format!("{} {}{}",
            self.name(),
            typename,
            if self.is_optional() { "" } else { " NOT NULL" },
        )
    }

    fn is_optional(&self) -> bool {
        let optional = match self {
            FieldDef::String(d) => d.optional,
            FieldDef::Integer(d) => d.optional,
            FieldDef::Float(d) => d.optional,
            FieldDef::Boolean(d) => d.optional,
            FieldDef::DateTime(d) => d.optional,
            FieldDef::User(d) => d.optional,
            FieldDef::Reference(d) => d.optional,
        };
        optional.unwrap_or(false)
    }
}

impl ModelDef {
    pub fn make_user_model() -> ModelDef {
        let fields = vec![
            FieldDef::make_string("name", "user name"),
            FieldDef::make_string("email", "user email"),
            FieldDef::make_string("pass", "user password"),
            FieldDef::make_string("token", "auth token"),
        ];
        ModelDef {
            name: "__oct_user".to_string(),
            description: Some("User model".to_string()),
            fields: Some(fields),
            visibility_scope: None,
        }
    }

    fn create_table_query(&self) -> String {
        let table_name = &self.name;
        let mut ret = format!(r#"CREATE TABLE {} (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    _oct_owner BIGINT,
    _oct_create_time timestamp NOT NULL DEFAULT (DATETIME('now')),
    _oct_update_time timestamp NOT NULL DEFAULT (DATETIME('now'))"#,
    table_name);

        for f in self.fields.as_ref().unwrap_or(&Vec::new()) {
            ret += ",\n    ";
            ret += f.type_sql().as_str();
        }
        ret += "\n);";
        ret
    }

    fn alter_table_query(&self, f: &FieldDef) -> String {
        format!(r#"ALTER TABLE {}
            ADD {}"#,
            self.name,
            f.type_sql()
        )
    }

    pub async fn create_table(&self, db: &DB) -> Result<()> {
        let sql = self.create_table_query();
        db.execute(&sql, &[])?;
        Ok(())
    }

    pub async fn alter_table(&self, db: &DB, old: &ModelDef) -> Result<()> {
        let old_fields: HashSet<&str> = if let Some(x) = &old.fields {
            x.into_iter().map(|f| f.name()).collect()
        } else {
            HashSet::new()
        };
        for f in self.fields.as_ref().unwrap_or(&Vec::new()) {
            if old_fields.contains(f.name()) {
                continue;
            }
            let sql = self.alter_table_query(f);
            db.execute(&sql, &[])?;
        }
        Ok(())
    }

    pub async fn create(&self, db: &DB, rec: &Row, uid: Option<i64>) -> Result<()> {
        let table_name = &self.name;
        let mut keys = vec!["_oct_owner"];
        let mut vals = vec![DbValue::Integer(uid.unwrap_or(-1))];
        for desc in self.fields.as_ref().unwrap() {
            let name = desc.name();
            let val = if !rec.fields.contains_key(name) {
                if let Some(x) = desc.default_value() {
                    x.to_db_value()
                } else {
                    if !desc.is_optional() {
                        bail!("Field {} is missing", name);
                    }
                    continue;
                }
            } else {
                rec.fields.get(name).unwrap().to_db_value()
            };
            keys.push(name);
            vals.push(val);
        }
        let sql = format!("INSERT INTO {} ({}) VALUES ({})",
                           table_name,
                           keys.join(","),
                           keys.iter().map(|_| "?").collect::<Vec<&str>>().join(","));
        db.execute(&sql, vals.as_slice());
        Ok(())
    }

    pub async fn update(&self, db: &DB, rec: &Row, uid: Option<i64>) -> Result<()> {
        let table_name = &self.name;
        let mut keys = Vec::new();
        let mut vals = Vec::new();
        let mut id = None;
        for (k, v) in rec.fields.iter() {
            if k == "id" {
                id = Some(v.get_int().unwrap());
                continue;
            }
            keys.push(k);
            vals.push(v.to_db_value());
        }
        let id = if let Some(x) = id {
            x
        } else {
            bail!("No id found in rec");
        };
        let sql = format!("UPDATE {} SET {} WHERE {}",
                           table_name,
                           keys.iter().map(|x| format!("{}=?", x)).collect::<Vec<String>>().join(","),
                           self.condition(uid, Some(id))
        );
        db.execute(&sql, vals.as_slice())?;
        Ok(())
    }


    pub async fn delete(&self, db: &DB, pks: &[i64], uid: Option<i64>) -> Result<usize> {
        let sql = format!("DELETE FROM {} WHERE {} AND id in ({})",
                           self.name,
                           self.condition(uid, None),
                           pks.iter().map(|x| x.to_string()).collect::<Vec<String>>().join(","));

        let r = db.execute(&sql, &[])?;
        Ok(r)
    }

    pub async fn select(&self, db: &DB, uid: Option<i64>, id: Option<i64>) -> Result<Vec<Row>> {
        db.query(self, &self.condition(uid, id))
    }

    fn get_visibility_scope(&self) -> ModelVisibilityScope {
        self.visibility_scope.clone().unwrap_or(ModelVisibilityScope::Everyone)
    }

    fn condition(&self, uid: Option<i64>, id: Option<i64>) -> String {
        let mut ret = match self.get_visibility_scope() {
            ModelVisibilityScope::Everyone => "1".to_string(),
            ModelVisibilityScope::Owner => {
                match uid {
                    None | Some(0) => "1".to_string(),
                    Some(x) => format!("_oct_owner={}", x),
                }
            },
        };
        if let Some(id) = id {
            ret += &format!(" AND id={}", id);
        }
        ret
    }
}
