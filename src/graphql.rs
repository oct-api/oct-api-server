use serde_json::{json, Value, map::Map};
use graphql_parser::query::*;
use crate::types::*;
use crate::db::DB;
use crate::http::*;

struct ExecuteContext<'a> {
    doc: Document<'a, String>,
    db: DB,
    app_def: AppDef,
    def: &'a GraphQLApiDesc,
    uid: Option<i64>,
}

impl<'s> ExecuteContext<'s> {
    async fn execute_doc(&self) -> Result<Value> {
        let mut data = Vec::new();
        for def in &self.doc.definitions {
            match def {
                Definition::Fragment(_) => { bail!("Fragment not supported"); }
                Definition::Operation(OperationDefinition::Query(query)) => {
                    data.push(self.execute_query(&query).await?);
                },
                Definition::Operation(OperationDefinition::SelectionSet(ss)) => {
                    for si in &ss.items {
                        data.push(self.execute_selection(si).await?);
                    }
                },
                x => {
                    bail!("Not supported: operation type is not query: {:?}", x);
                }
            }
        };
        Ok(json!({
            "data": data,
        }))
    }

    async fn execute_selection<'a>(&self, selection: &Selection<'a, String>) -> Result<Value> {
        match selection {
            Selection::Field(field) => {
                self.query_field(&field).await
            },
            _ => {
                bail!("Not supported: selection type is not field");
            }
        }

    }

    async fn execute_query<'a>(&self, query: &Query<'a, String>) -> Result<Value> {
        let mut ret = Vec::new();
        if query.variable_definitions.len() > 0 {
            bail!("Variable definitions not supported");
        }
        if query.directives.len() > 0 {
            bail!("Directives not supported");
        }

        for si in &query.selection_set.items {
            ret.push(self.execute_selection(si).await?);
        }
        Ok(serde_json::to_value(ret)?)
    }

    async fn query_field<'a>(&self, field: &Field<'a, String>) -> Result<Value> {
        let mut ret = Vec::new();
        let field_name = &field.name;

        if field.alias.is_some() {
            bail!("Field alias is not supported");
        }
        if field.arguments.len() > 0 {
            bail!("Field arguments is not supported");
        }
        if field.directives.len() > 0 {
            bail!("Field directives is not supported");
        }
        let model = if let Some(m) = self.app_def.get_model(field_name) {
            m
        } else {
            bail!("Model {} not found", field_name);
        };
        for rec in model.select(&self.db, self.uid, None).await? {
            let mut map = serde_json::map::Map::new();
            for si in &field.selection_set.items {
                match si {
                    Selection::Field(f) => {
                        if f.selection_set.items.len() > 0 {
                            bail!("Nested selection not supported");
                        }
                        if let Some(x) = rec.get(&f.name) {
                            map.insert(f.name.clone(), x.to_value());
                        } else {
                            bail!("f '{}' not found, fields are {:?}",
                                  f.name, rec.fields);
                        }
                    },
                    _ => {
                        bail!("Selection type not supported");
                    }
                }
            }
            ret.push(json!(map));
        }
        Ok(json!(&ret))
    }
}

fn parse_query<'a>(query: &'a str) -> Result<Document<'a, String>> {
    let r = graphql_parser::query::parse_query::<String>(query)?;
    Ok(r)
}

pub async fn handle_graphql_get(req: Request,
                                app: &OctApp,
                                def: &GraphQLApiDesc,
                                uid: Option<i64>) -> Result<Response> {
    let qm = get_query(&req);
    let query = if let Some(x) = qm.get("query") {
        x
    } else {
        return http400("No GraphQL query in request");
    };
    let doc = match parse_query(query) {
        Ok(x) => x,
        Err(e) => {
            return http400(&format!("Invalid query: {}", e));
        },
    };
    let app_def = if let Some(x) = app.get_def().await {
        x
    } else {
        bail!("Cannot get app def");
    };
    let exec_ctx = ExecuteContext {
        doc,
        app_def,
        db: app.db()?,
        def,
        uid,
    };
    match exec_ctx.execute_doc().await {
        Ok(x) => json_response(&x),
        Err(e) => http400(&format!("Failed to execute graphql request: {}", e)),
    }
}

pub async fn handle_graphql_post(req: Request,
                                 app: &OctApp,
                                 def: &GraphQLApiDesc,
                                 uid: Option<i64>) -> Result<Response> {
    http404("not implemented")
}

pub async fn handle_graphql(req: Request,
                            app: &OctApp,
                            def: &GraphQLApiDesc,
                            uid: Option<i64>) -> Result<Response> {
    match req.method() {
        &hyper::Method::GET => handle_graphql_get(req, app, def, uid).await,
        &hyper::Method::POST => handle_graphql_post(req, app, def, uid).await,
        _ => bail!("Method not supported for graphql"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile;
    use crate::apps::sync_models;

    fn read_data(name: &str) -> String {
        fs::read_to_string(&format!("tests/data/{}", name)).unwrap()
    }

    async fn do_test(app: &str, data: &str, query: &str, result: &str) {
        let qs = read_data(query);
        let doc = parse_query(&qs).unwrap();
        let yml = read_data(app);
        let app_def = AppDef::from_yaml(&yml).unwrap();
        let tf = tempfile::NamedTempFile::new().unwrap();
        let dbpath = tf.path();
        let db = DB::new(dbpath.to_str().unwrap()).unwrap();
        let ep = app_def.api.find_endpoint("/graphql").unwrap();
        sync_models(&db, &None, &app_def).await.unwrap();
        let v: Value = serde_json::from_str(&read_data(data)).unwrap();
        prepare_data(&db, &app_def, &v).await;
        let def = match &ep {
            ApiEndpoint::GraphQL(x) => x,
            _ => { panic!("invalid type"); }
        };
        let exec_ctx = ExecuteContext {
            doc,
            app_def,
            db,
            def,
            uid: None,
        };
        let r = exec_ctx.execute_doc().await.unwrap();
        let expected: Value = serde_json::from_str(&read_data(result)).unwrap();
        assert_eq!(r, expected);
    }

    async fn prepare_data(db: &DB, app_def: &AppDef, data: &Value) {
        for (k, vs) in data.as_object().unwrap() {
            for v in vs.as_array().unwrap() {
                let model = app_def.get_model(k).unwrap();
                let rec = Row::from_value(&v).unwrap();
                model.create(db, &rec, None).await.unwrap();
            }
        }
    }

    #[tokio::test]
    async fn simple_test() {
        do_test("graphql.yml",
                "graphql-01-data.json",
                "graphql-01-query.txt",
                "graphql-01-result.json")
            .await;
    }
}
