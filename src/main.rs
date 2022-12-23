mod config;
mod http;
mod types;
mod auth;
mod stor;
mod meta;
mod apps;
mod api;
mod model;
mod db;
mod worker;
mod orm;
mod stats;
mod graphql;
mod alert;

use std::sync::Arc;
use futures::try_join;
use crate::types::*;

#[macro_use]
extern crate lazy_static;

fn start_ui() {
    println!("Starting ui...");
    std::process::Command::new("npm")
        .current_dir("ui")
        .args(&vec!["run", "serve"])
        .spawn()
        .expect("failed to start ui dev server");
}

fn start_doc() {
    println!("Starting doc...");
    std::process::Command::new("mkdocs")
        .current_dir("doc")
        .args(&vec!["serve"])
        .spawn()
        .expect("failed to start doc dev server");
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = clap::App::new("Oct API server")
        .version("1.0")
        .author("Fam Zheng <fam@euphon.net>")
        .about("The Oct API server program")
        .arg(clap::Arg::with_name("addr")
            .long("--addr")
            .short("-a")
            .takes_value(true)
            .help("Address to listen to"))
        .arg(clap::Arg::with_name("data")
            .long("--data")
            .short("-d")
            .takes_value(true)
            .help("Data directory"))
        .arg(clap::Arg::with_name("start-ui")
            .long("--start-ui")
            .short("-U")
            .help("Start ui dev server"))
        .arg(clap::Arg::with_name("orm-addr")
            .long("--orm-addr")
            .short("-J")
            .takes_value(true)
            .help("Django server address"))
        .arg(clap::Arg::with_name("no-start-orm")
            .long("--no-start-orm")
            .help("Don't start Django server"))
        .arg(clap::Arg::with_name("start-doc")
            .long("--start-doc")
            .short("-D")
            .help("Start doc dev server"))
        .get_matches();
    {
        let mut cfg = config::config_write();
        if let Some(x) = matches.value_of("addr") {
            cfg.server_addr = x.to_string();
        }
        if let Some(x) = matches.value_of("data") {
            cfg.data_dir = x.to_string();
        }
        if let Some(x) = matches.value_of("orm-addr") {
            cfg.orm_addr = x.to_string();
        }
    }
    let stats = stats::try_load_stats().await;
    let ctx = Arc::new(Context::new(stats));
    if !matches.is_present("no-start-orm") {
        orm::start().await.expect("Cannot start ORM");
    }
    if matches.is_present("start-ui") {
        start_ui();
    }
    if matches.is_present("--start-doc") {
        start_doc();
    }
    let server = http::run_server(ctx.clone());
    let worker = worker::run_worker(ctx.clone());

    try_join!(server, worker)?;
    Ok(())
}
