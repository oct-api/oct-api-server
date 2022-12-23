use std::fmt;
use std::sync::{Mutex, MutexGuard};
pub use std::collections::{HashMap, VecDeque};
pub use std::sync::Arc;
use chrono::{Utc, Timelike, Duration};
use hyper;
use rand::Rng;
pub use anyhow::{anyhow, bail};
pub use serde::{Serialize, Deserialize};
use serde_json::{Value, Number};
pub use crate::config::config;

pub type Result<T> = anyhow::Result<T>;
pub type Error = anyhow::Error;
pub type Request = hyper::Request<hyper::Body>;
pub type Response = hyper::Response<hyper::Body>;

pub type DateTime = chrono::DateTime<Utc>;

#[derive(Debug)]
pub enum TimeSeriesUnit {
    Minutely,
    Hourly,
    Daily,
}

impl TimeSeriesUnit {
    fn len(&self) -> usize {
        match self {
            Self::Minutely => 60,
            Self::Hourly => 24,
            Self::Daily => 30,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TimeSeriesData {
    data: VecDeque<u64>,
    last_interval_start: Option<DateTime>,
    capacity: usize,
    interval_secs: i64,
}

impl TimeSeriesData {
    fn new(capacity: usize, interval: Duration) -> TimeSeriesData {
        TimeSeriesData {
            data: VecDeque::new(),
            last_interval_start: None,
            capacity,
            interval_secs: interval.num_seconds(),
        }
    }

    fn interval(&self) -> Duration {
        Duration::seconds(self.interval_secs)
    }

    fn sum(&self) -> u64 {
        self.data.iter().sum()
    }

    fn push_raw(&mut self, val: u64) {
        self.data.push_front(val);
        while self.data.len() > self.capacity {
            self.data.pop_back();
        }
    }

    fn get_data(&self) -> Vec<u64> {
        self.data.iter().map(|x| x.clone()).collect()
    }

    fn add_data(&mut self, val: u64) {
        let now = now();
        if self.last_interval_start.is_none() {
            self.last_interval_start = Some(now);
            self.push_raw(val);
            return;
        }
        let interval = self.interval();
        let mut t = self.last_interval_start.unwrap();
        while now - t > Duration::seconds(2 * self.interval_secs) {
            /* Fill the gap where we have no data */
            self.push_raw(0);
            t = t.checked_add_signed(interval).unwrap();
        }
        if now - t > interval {
            self.push_raw(val);
            t = t.checked_add_signed(interval).unwrap();
            self.last_interval_start = Some(t);
        } else {
            let front = self.data.get_mut(0).unwrap();
            *front += val;
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TimeSeries {
    minutely: TimeSeriesData,
    hourly: TimeSeriesData,
    daily: TimeSeriesData,
}

pub fn now() -> DateTime {
    Utc::now()
}

impl TimeSeries {
    fn new() -> TimeSeries {
        TimeSeries {
            minutely: TimeSeriesData::new(60, Duration::minutes(1)),
            hourly: TimeSeriesData::new(24, Duration::hours(1)),
            daily: TimeSeriesData::new(30, Duration::days(1)),
        }
    }

    fn add_data(&mut self, n: u64) {
        self.minutely.add_data(n);
        self.hourly.add_data(n);
        self.daily.add_data(n);
    }

    fn get_minutely(&self) -> Vec<u64> {
        self.minutely.get_data()
    }

    fn get_hourly(&self) -> Vec<u64> {
        self.hourly.get_data()
    }

    fn get_daily(&self) -> Vec<u64> {
        self.daily.get_data()
    }

    fn get_by_unit(&self, unit: &TimeSeriesUnit) -> Vec<u64> {
        match unit {
            TimeSeriesUnit::Minutely => self.get_minutely(),
            TimeSeriesUnit::Hourly => self.get_hourly(),
            TimeSeriesUnit::Daily => self.get_daily(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    counters: HashMap<String, u64>,
    series_map: HashMap<String, TimeSeries>,
    last_tick: DateTime,
}

impl Stats {
    pub fn new() -> Stats {
        Stats {
            counters: HashMap::new(),
            series_map: HashMap::new(),
            last_tick: now(),
        }
    }

    pub fn account(&mut self, evt: &str) {
        match self.counters.get_mut(evt) {
            Some(v) => { *v += 1 },
            None => { self.counters.insert(evt.to_string(), 1); }
        }
    }

    pub fn get_by_prefix(&self, pref: &str) -> HashMap<String, u64> {
        let mut ret = HashMap::new();
        for (k, v) in self.counters.iter() {
            if k.starts_with(pref) {
                ret.insert(k.to_string(), *v);
            }
        }
        ret
    }

    /* Periodic tick callback to shift current counters into time series. */
    pub fn tick(&mut self) {
        let now = now();
        if self.last_tick.minute() == now.minute() {
            return;
        }
        for (k, v) in self.counters.iter_mut() {
            if ! self.series_map.contains_key(k) {
                self.series_map.insert(k.to_string(), TimeSeries::new());
            }
            self.series_map
                .get_mut(k)
                .unwrap()
                .add_data(*v);
            *v = 0;
        }
        self.last_tick = now;
    }

    pub fn time_series_by_prefix(&self, pref: &str, unit: &TimeSeriesUnit) -> Vec<u64> {
        let mut ret = vec![0; unit.len()];
        ret[0] = self.get_by_prefix(pref).values().sum();
        for (k, v) in self.series_map.iter() {
            if !k.starts_with(pref) {
                continue;
            }
            let ts = v.get_by_unit(unit);
            for (i, v) in ts.iter().enumerate() {
                if i >= ret.len() {
                    break;
                }
                ret[i] += v;
            }
        }
        ret
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    _stats: Mutex<Stats>,
}

impl Context {
    pub fn new(stats: Stats) -> Context {
        Context {
            _stats: Mutex::new(stats),
        }
    }

    pub fn stats(&self) -> MutexGuard<Stats> {
        self._stats.lock().unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OctApp {
    pub id: Option<i64>,
    pub user: Option<i64>,
    pub name: String,
    pub handle: String,
    pub git_repo: Option<String>,
    pub git_ref: Option<String>,
    pub admin_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum LoginType {
    Github,
}

impl LoginType {
    fn parse(s: &str) -> Option<LoginType> {
        match s {
            "github" => Some(LoginType::Github),
            _ => None
        }
    }
}

impl fmt::Display for LoginType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let r = match self {
            Self::Github => "github",
        };
        write!(f, "{}", r)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OctUser {
    pub id: Option<i64>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub token: String,
}

impl OctUser {
    pub fn new(username: &str, display_name: &str, email: &str) -> OctUser {
        OctUser {
            id: None,
            username: username.to_string(),
            display_name: display_name.to_string(),
            email: email.to_string(),
            token: gen_random_string(24),
        }
    }
}

fn validate_id(id: &str) -> Result<()> {
    if id.len() < 1 {
        bail!("empty identifier");
    }
    for c in id.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => (),
            c => { bail!("invalid char: '{}'", c); }
        }
    }
    Ok(())
}

fn validate_text(text: &str, max_length: usize) -> Result<()> {
    if text.len() > max_length {
        bail!("text too long");
    }
    for c in text.chars() {
        if c.is_control() {
            bail!("invalid control char in string");
        }
    }
    Ok(())
}

fn validate_api_path(path: &str) -> Result<()> {
    if path.len() < 1 {
        bail!("empty api path");
    }
    for c in path.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '/' | '-' | '.' => (),
            c => { bail!("invalid char: '{}'", c); }
        }
    }
    Ok(())
}

fn validate_file_path(path: &str) -> Result<()> {
    if path.len() < 1 {
        bail!("empty api path");
    }
    for c in path.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' | '/' | '-' | '.' => (),
            c => { bail!("invalid char: '{}'", c); }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SimpleDesc {
    pub name: String,
    pub description: Option<String>,
    pub optional: Option<bool>
}

impl SimpleDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DateTimeDesc {
    pub name: String,
    pub description: Option<String>,
    pub optional: Option<bool>,
    pub default_now: Option<bool>,
}

impl DateTimeDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceDesc {
    pub name: String,
    pub description: Option<String>,
    pub target: String,
    pub optional: Option<bool>
}

impl ReferenceDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        validate_id(&self.target)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum FieldDef {
    String(SimpleDesc),
    Integer(SimpleDesc),
    Boolean(SimpleDesc),
    Float(SimpleDesc),
    DateTime(DateTimeDesc),
    User(SimpleDesc),
    Reference(ReferenceDesc),
}

impl FieldDef {
    fn validate(&self) -> Result<()> {
        match self {
            Self::String(d) => d.validate()?,
            Self::Integer(d) => d.validate()?,
            Self::Boolean(d) => d.validate()?,
            Self::Float(d) => d.validate()?,
            Self::User(d) => d.validate()?,
            Self::DateTime(d) => d.validate()?,
            Self::Reference(d) => d.validate()?,
        }
        Ok(())
    }
    pub fn make_string(name: &str, desc: &str) -> FieldDef {
        FieldDef::String(
            SimpleDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
            }
        )
    }
    pub fn make_integer(name: &str, desc: &str) -> FieldDef {
        FieldDef::Integer(
            SimpleDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
            }
        )
    }
    pub fn make_boolean(name: &str, desc: &str) -> FieldDef {
        FieldDef::Boolean(
            SimpleDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
            }
        )
    }
    pub fn make_float(name: &str, desc: &str) -> FieldDef {
        FieldDef::Float(
            SimpleDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
            }
        )
    }
    pub fn make_datetime(name: &str, desc: &str) -> FieldDef {
        FieldDef::DateTime(
            DateTimeDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
                default_now: None,
            }
        )
    }
    pub fn make_user(name: &str, desc: &str) -> FieldDef {
        FieldDef::User(
            SimpleDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                optional: None,
            }
        )
    }
    pub fn make_reference(name: &str, desc: &str, target: &str) -> FieldDef {
        FieldDef::Reference(
            ReferenceDesc {
                name: name.to_string(),
                description: Some(desc.to_string()),
                target: target.to_string(),
                optional: None,
            }
        )
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ApiAccessRuleAction {
    Allow,
    Deny,
}

impl ApiAccessRuleAction {
    pub fn allowed(&self) -> bool {
        self == &Self::Allow
    }
}

fn default_action() -> ApiAccessRuleAction {
    ApiAccessRuleAction::Allow
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ApiAccessMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

impl ApiAccessMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Post => "post",
            Self::Put => "put",
            Self::Delete => "delete",
            Self::Patch => "patch",
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ApiAccessRuleDef {
    #[serde(default = "default_action")]
    pub action: ApiAccessRuleAction,
    pub method: Option<ApiAccessMethod>,
    pub role: Option<String>,
}

impl ApiAccessRuleDef {
    fn validate(&self) -> Result<()> {
        if let Some(role) = &self.role {
            validate_id(role)?;
        }
        Ok(())
    }

    fn simple(action: ApiAccessRuleAction) -> ApiAccessRuleDef {
        ApiAccessRuleDef {
            action,
            method: None,
            role: None,
        }
    }

    fn allow_aonnymous_get() -> ApiAccessRuleDef {
        ApiAccessRuleDef {
            action: ApiAccessRuleAction::Allow,
            role: None,
            method: Some(ApiAccessMethod::Get)
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StringApiDesc {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub response: String,
    pub access: Option<Vec<ApiAccessRuleDef>>,
}

impl StringApiDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        validate_api_path(&self.path)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        validate_text(&self.response, 4096)?;
        if let Some(x) = &self.access {
            for d in x {
                d.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct StaticApiDesc {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub localfile: String,
    pub access: Option<Vec<ApiAccessRuleDef>>,
}

impl StaticApiDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        validate_api_path(&self.path)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        validate_file_path(&self.localfile)?;
        if let Some(x) = &self.access {
            for d in x {
                d.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct ModelApiDesc {
    pub name: String,
    pub path: String,
    pub description: Option<String>,
    pub model: String,
    pub access: Option<Vec<ApiAccessRuleDef>>,
}

impl ModelApiDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        validate_api_path(&self.path)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        validate_id(&self.model)?;
        if let Some(x) = &self.access {
            for d in x {
                d.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct GraphQLApiDesc {
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub access: Option<Vec<ApiAccessRuleDef>>,
}

impl GraphQLApiDesc {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        validate_api_path(&self.path)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        if let Some(x) = &self.access {
            for d in x {
                d.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ModelVisibilityScope {
    Everyone,
    Owner,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ModelDef {
    pub name: String,
    pub description: Option<String>,
    pub fields: Option<Vec<FieldDef>>,
    pub visibility_scope: Option<ModelVisibilityScope>,
}

impl ModelDef {
    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        if let Some(x) = &self.description {
            validate_text(x, 1024)?;
        }
        if let Some(x) = &self.fields {
            for f in x {
                f.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ApiEndpoint {
    String(StringApiDesc),
    StaticFile(StaticApiDesc),
    Model(ModelApiDesc),
    GraphQL(GraphQLApiDesc),
}

impl ApiEndpoint {
    pub fn get_path(&self) -> String {
        let r = match self {
            ApiEndpoint::String(x) => &x.path,
            ApiEndpoint::StaticFile(x) => &x.path,
            ApiEndpoint::Model(x) => &x.path,
            ApiEndpoint::GraphQL(x) => &x.path,
        };
        String::from(r)
    }

    pub fn name(&self) -> &str {
        match self {
            ApiEndpoint::String(x) => &x.name,
            ApiEndpoint::StaticFile(x) => &x.name,
            ApiEndpoint::Model(x) => &x.name,
            ApiEndpoint::GraphQL(x) => &x.name,
        }
    }

    pub fn access(&self) -> Option<&Vec<ApiAccessRuleDef>> {
        let x = match self {
            ApiEndpoint::String(x) => &x.access,
            ApiEndpoint::StaticFile(x) => &x.access,
            ApiEndpoint::Model(x) => &x.access,
            ApiEndpoint::GraphQL(x) => &x.access,
        };
        x.as_ref()
    }

    fn validate(&self) -> Result<()> {
        match self {
            ApiEndpoint::String(x) => x.validate()?,
            ApiEndpoint::StaticFile(x) => x.validate()?,
            ApiEndpoint::Model(x) => x.validate()?,
            ApiEndpoint::GraphQL(x) => x.validate()?,
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum ApiDefAccessDef {
    Action(ApiAccessRuleAction),
    Rules(Vec<ApiAccessRuleDef>),
}

impl ApiDefAccessDef {
    pub fn get_rules(&self) -> Vec<ApiAccessRuleDef> {
        match self {
            Self::Action(action) =>
                vec![ApiAccessRuleDef::simple(action.clone())],
            Self::Rules(rules) => rules.clone(),
        }
    }

    pub fn default() -> Vec<ApiAccessRuleDef> {
        vec![ApiAccessRuleDef::allow_aonnymous_get()]
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ApiDef {
    pub default_access: Option<ApiDefAccessDef>,
    pub endpoints: Vec<ApiEndpoint>,
}

impl ApiDef {
    pub fn get_default_access(&self) -> Vec<ApiAccessRuleDef> {
        match &self.default_access {
            Some(x) => x.get_rules(),
            _ => ApiDefAccessDef::default(),
        }
    }
}

impl ApiDef {
    pub fn find_endpoint(&self, path: &str) -> Option<ApiEndpoint> {
        for ep in &self.endpoints {
            if ep.get_path() == path {
                return Some(ep.clone())
            }
        }
        None
    }

    fn validate(&self) -> Result<()> {
        for ep in &self.endpoints {
            ep.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AppEvent {
    pub app: i64,
    pub datetime: Option<DateTime>,
    pub content: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AppMeta {
    schema: String,
}

impl AppMeta {
    fn validate(&self) -> Result<()> {
        if self.schema != "v0.0.1" {
            bail!("Unsupported schema version: {}", self.schema);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AppDef {
    pub name: String,
    pub models: Vec<ModelDef>,
    pub api: ApiDef,
    pub meta: AppMeta,
}

impl AppDef {
    pub fn from_yaml(yml: &str) -> Result<AppDef> {
        let app: AppDef = serde_yaml::from_str(yml)?;
        app.validate()?;
        Ok(app)
    }

    fn validate(&self) -> Result<()> {
        validate_id(&self.name)?;
        self.meta.validate()?;
        for model in &self.models {
            model.validate()?;
        }
        self.api.validate()?;
        Ok(())
    }

    pub fn get_model(&self, model: &str) -> Option<&ModelDef> {
        for m in &self.models {
            if m.name == model {
                return Some(m);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum RowField {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    DateTime(i64),
    Null,
}

impl RowField {
    pub fn to_value(&self) -> Value {
        match self {
            Self::String(v) => Value::String(v.to_string()),
            Self::Integer(v) => Value::Number(Number::from(*v)),
            Self::Float(v) => Value::Number(Number::from_f64(*v).unwrap()),
            Self::DateTime(v) => Value::Number(Number::from(*v)),
            Self::Boolean(v) => Value::Bool(*v),
            Self::Null => Value::Null,
        }
    }

    pub fn get_int(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub fields: HashMap<String, RowField>,
}

impl Row {
    pub fn new() -> Row {
        Row {
            fields: HashMap::new()
        }
    }

    pub fn from_json(s: &str) -> Result<Row> {
        let v = serde_json::from_str(s)?;
        Self::from_value(&v)
    }

    pub fn from_value(v: &Value) -> Result<Row> {
        let mut ret = Row::new();
        match v {
            Value::Object(o) => {
                for (k, v) in o.iter() {
                    let f = match v {
                        Value::Bool(x) => RowField::Boolean(*x),
                        Value::Number(x) => {
                            if x.is_f64() {
                                RowField::Float(x.as_f64().unwrap())
                            } else {
                                let n = x.as_i64().ok_or(anyhow!("Invalid input"))?;
                                RowField::Integer(n)
                            }
                        }
                        Value::String(x) => RowField::String(x.to_string()),
                        _ => bail!("Invalid data type"),
                    };
                    ret.fields.insert(k.to_string(), f);
                }
            },
            _ => bail!("Invalid data"),
        }
        Ok(ret)
    }

    pub fn to_value(&self) -> Value {
        let mut ret = serde_json::map::Map::new();
        for (k, v) in &self.fields {
            ret.insert(k.to_string(), v.to_value());
        }
        Value::Object(ret)
    }

    pub fn get(&self, field: &str) -> Option<&RowField> {
        self.fields.get(field)
    }

    pub fn set(&mut self, field: &str, val: RowField) {
        self.fields.insert(field.to_string(), val);
    }

    pub fn get_str(&self, field: &str) -> Option<&str> {
        match self.fields.get(field) {
            Some(RowField::String(s)) => Some(&s),
            _ => None,
        }
    }

    pub fn get_int(&self, field: &str) -> Option<i64> {
        match self.fields.get(field) {
            Some(RowField::Integer(s)) => Some(*s),
            _ => None,
        }
    }
}

pub fn gen_random_string(n: usize) -> String {
    let mut ret = String::new();
    let mut rng = rand::thread_rng();
    for _ in 0..n {
        ret += &std::char::from_u32('A' as u32 + rng.gen_range(0..26)).unwrap().to_string();
    }
    ret
}

#[test]
fn stats_test() {
    let mut stats = Stats::new();
    stats.account("hello.world");
    stats.last_tick = now() - Duration::seconds(61);
    stats.tick();
    let d = stats.time_series_by_prefix("hello.", &TimeSeriesUnit::Minutely);
    assert_eq!(d.len(), 60);
    assert_eq!(d[0], 1);
    let d = stats.time_series_by_prefix("hello.", &TimeSeriesUnit::Hourly);
    assert_eq!(d.len(), 24);
    assert_eq!(d[0], 1);
    stats.account("hello.world");
    let d = stats.time_series_by_prefix("hello.", &TimeSeriesUnit::Daily);
    assert_eq!(d.len(), 30);
    assert_eq!(d[0], 2);
}

#[test]
fn validate_meta_test() {
    // Positive and negative cases of meta schema version string
    let bad = "
meta:
  schema: v2
name: test
models: []
api:
  endpoints: []
      ";
    let r = AppDef::from_yaml(bad);
    assert!(r.is_err());
    let good = "
meta:
  schema: v0.0.1
name: test
models: []
api:
  endpoints: []
      ";
    let r = AppDef::from_yaml(good);
    assert!(r.is_ok());
}
