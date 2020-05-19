#![warn(rust_2018_idioms)]

use clap::{App, AppSettings, Arg};
use exitfailure::ExitFailure;
use git2::Repository;
use log::info;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::env;

#[derive(Debug, Deserialize)]
struct Parent {
    ssh_url: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    name: String,
    parent: Parent,
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let matches = App::new("github-cli-utils")
        .version("0.1")
        .author("Daiki Ihara <sasurau4@gmail.com>")
        .about("Support developing OSS via cli")
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .multiple(true)
                .global(true)
                .takes_value(false)
                .about("Turn debugging information on"),
        )
        .subcommand(App::new("add-upstream").about("add upstream to your git repository"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();
    let cwd = env::current_dir()?;
    info!("cwd: {:#?}", &cwd);

    let log_level = match matches.occurrences_of("debug") {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };
    env_logger::builder().filter_level(log_level).init();
    info!("log_level: {}", log_level);

    match matches.subcommand_name() {
        Some("add-upstream") => {
            let repo = Repository::open(&cwd).unwrap();
            let origin = repo.find_remote("origin").unwrap();
            let repo_url = origin.url().unwrap();
            let splitted_repo_url: Vec<_> = repo_url.split("github.com/").collect();
            let splitted_git_url: Vec<_> = splitted_repo_url[1].split(".git").collect();
            let github_repo_name = splitted_git_url[0];
            info!("full_name: {:#?}", github_repo_name);

            let resp = get_repository_info(github_repo_name).await?;
            info!("users: {:#?}", resp);

            repo.remote_set_url("upstream", &resp.parent.ssh_url)
                .unwrap();
        }
        None => unreachable!(),
        _ => unreachable!(),
    }

    Ok(())
}

async fn get_repository_info(repo_name: &str) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo_name);
    let response = client
        .get(&url)
        .header(USER_AGENT, "github-cli-utils")
        .send()
        .await?;
    match response.error_for_status() {
        Ok(res) => res.json::<Response>().await,
        Err(err) => panic!("Response failed: {}", err),
    }
}
