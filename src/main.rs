use anyhow::Result;
use clap::{App, AppSettings, Arg, ArgAction};
use git2::Repository;
use log::{debug, error};
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::env;
use thiserror::Error;

#[derive(Debug, Deserialize)]
struct Parent {
    ssh_url: String,
}

#[derive(Debug, Deserialize)]
struct Response {
    parent: Parent,
}

#[derive(Error, Debug)]
enum GithubCliUtilError {
    #[error("Repository Setting is invalid: {0}")]
    RepositorySettingError(String),
    #[error("GitHub api response fail: {0}")]
    GitHubApiError(String),
}

impl From<reqwest::Error> for GithubCliUtilError {
    fn from(e: reqwest::Error) -> GithubCliUtilError {
        GithubCliUtilError::GitHubApiError(format!("{:?}", e))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = App::new("github-cli-utils")
        .version("0.1.1")
        .author("Daiki Ihara <sasurau4@gmail.com>")
        .about("Support developing OSS via cli")
        .arg(
            Arg::new("debug")
                .long("debug")
                .short('d')
                .global(true)
                .takes_value(false)
                .action(ArgAction::Count)
                .help("Turn debugging information on"),
        )
        .subcommand(App::new("add-upstream").about("add upstream to your git repository"))
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .get_matches();
    let cwd = env::current_dir()?;
    debug!("cwd: {:#?}", &cwd);

    let log_level = match matches
        .get_one::<u8>("debug")
        .expect("Count's are defaulted")
    {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        _ => log::LevelFilter::Debug,
    };
    env_logger::builder().filter_level(log_level).init();
    debug!("log_level: {}", log_level);

    match matches.subcommand_name() {
        Some("add-upstream") => {
            let repo = Repository::open(&cwd).unwrap();
            let origin = repo.find_remote("origin").unwrap();
            let repo_url = origin.url().unwrap();
            let splitted_repo_url: Vec<_> = repo_url.split("github.com/").collect();
            let splitted_git_url: Vec<_> = splitted_repo_url[1].split(".git").collect();
            let github_repo_name = splitted_git_url[0];
            debug!("target_repo_name: {:#?}", github_repo_name);

            let resp = get_repository_info(github_repo_name).await?;

            let upstream_url = format!("ssh://{}", resp.parent.ssh_url);
            debug!("upstream_url: {}", upstream_url);
            repo.remote_set_url("upstream", &upstream_url).unwrap();
        }
        None => unreachable!(),
        _ => unreachable!(),
    }

    Ok(())
}

async fn get_repository_info(repo_name: &str) -> Result<Response, GithubCliUtilError> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo_name);
    let response = client
        .get(&url)
        .header(USER_AGENT, "github-cli-utils")
        .send()
        .await?;
    match response.error_for_status() {
        Ok(res) => {
            let result = res.json::<Response>().await.map_err(|e| {
                error!("Convert response to json struct is fail: {:?}", e);
                GithubCliUtilError::RepositorySettingError(
                    "This repository doesn't have parent repository".to_string(),
                )
            });
            result
        }
        Err(err) => {
            error!("GitHub api response is fail: {:?}", err);
            Err(GithubCliUtilError::GitHubApiError(format!("{:?}", err)))
        }
    }
}
