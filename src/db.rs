use file_lock::FileLock;
use rusqlite::{types::*, Connection};
use crate::types::*;

pub type DbValue = rusqlite::types::Value;

#[derive(Debug)]
pub struct DB {
    conn: Connection,
    lock: FileLock,
}

// FIXME: thread safety of underlying db api
unsafe impl Send for DB {}
unsafe impl Sync for DB {}

impl DB {
    pub fn new(path: &str) -> Result<DB> {
        let conn = Connection::open(path)?;
        let lock = FileLock::lock(path, true, true)?;
        Ok(DB {
            conn,
            lock,
        })
    }

    pub async fn tables(&self) -> Result<Vec<String>> {
        let sql = "SELECT name FROM sqlite_master WHERE type='table'";
        let mut stmt = self.conn.prepare(sql)?;
        let mut ret = Vec::new();
        let rows = stmt.query_map([], |row| row.get(0))?;
        for row in rows {
            ret.push(row?);
        }
        Ok(ret)
    }

    fn do_query(&self, model: &ModelDef, cond: Option<&str>, max: Option<usize>) -> Result<Vec<Row>> {
        let fields = model.fields.as_ref().unwrap_or(&Vec::new()).iter().map(|f| f.name()).collect::<Vec<_>>().join(",");
        let mut stmt = self.conn.prepare(&format!("SELECT id, {} FROM {} WHERE {}",
                fields, model.name, cond.unwrap_or("1")))?;
        let rows = stmt.query_map([], |r| {
            let mut row = Row::new();
            for (fi, f) in model.fields.as_ref().unwrap_or(&Vec::new()).iter().enumerate() {
                let i = fi + 1; // col 0 is id
                match r.get_ref(i) {
                    Ok(ValueRef::Null) => {
                        row.fields.insert(f.name().to_string(), RowField::Null);
                        continue;
                    }
                    _ => {}
                }
                let rf = match f {
                    FieldDef::String(_) => RowField::String(r.get(i)?),
                    FieldDef::Integer(_) => RowField::Integer(r.get(i)?),
                    FieldDef::Boolean(_) => RowField::Boolean(r.get(i)?),
                    FieldDef::Float(_) => RowField::Float(r.get(i)?),
                    FieldDef::DateTime(_) => RowField::DateTime(r.get(i)?),
                    FieldDef::User(_) => RowField::Integer(r.get(i)?),
                    FieldDef::Reference(_) => RowField::Integer(r.get(i)?),
                };
                row.fields.insert(f.name().to_string(), rf);
            }
            row.fields.insert("id".to_string(), RowField::Integer(r.get(0)?));
            Ok(row)
        })?;
        let mut ret = Vec::new();
        let mut cnt = 0;
        for row in rows {
            let r = match row {
                Ok(x) => x,
                Err(e) => { println!("err: {}", e); bail!("error") },
            };
            ret.push(r);
            if let Some(x) = max {
                if cnt >= x {
                    break;
                }
            }
            cnt += 1;
        }
        Ok(ret)
    }

    pub fn query(&self, model: &ModelDef, cond: &str) -> Result<Vec<Row>> {
        self.do_query(model, Some(cond), None)
    }

    pub fn get(&self, model: &ModelDef, cond: &str) -> Result<Option<Row>> {
        let mut r = self.do_query(model, Some(cond), Some(1))?;
        Ok(r.pop())
    }

    pub fn execute(&self, sql: &str, values: &[DbValue]) -> Result<usize> {
        let vals: Vec<&dyn ToSql> = values.iter().map(|x| x as &dyn ToSql).collect();
        let r = self.conn.execute(&sql, vals.as_slice())?;
        Ok(r)
    }
}

impl RowField {
    pub fn to_db_value(&self) -> DbValue {
        match self {
            Self::String(v) => DbValue::Text(v.to_string()),
            Self::Integer(v) => DbValue::Integer(*v),
            Self::Float(v) => DbValue::Real(*v),
            Self::Boolean(v) => DbValue::Integer(if *v { 1 } else { 0 }),
            Self::DateTime(v) => DbValue::Integer(*v),
            Self::Null => DbValue::Null,
        }
    }
}
