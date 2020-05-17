#![warn(rust_2018_idioms)]

use git2::Repository;
use reqwest::header::USER_AGENT;
use serde::Deserialize;
use std::env;
// use failure::ResultExt;
use clap::App;

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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("github-cli-utils")
        .version("0.1")
        .author("Daiki Ihara <sasurau4@gmail.com>")
        .about("Support developing OSS via cli")
        .arg("-d... 'Turn Debuggging information on'")
        .subcommand(App::new("add-upstream").about("add upstream to your git repository"))
        .get_matches();
    let cwd = env::current_dir()?;
    println!("cwd: {:#?}", &cwd);

    println!("matches: {:#?}", matches);
    match matches.subcommand_name() {
        Some("add-upstream") => {
            let repo = Repository::open(&cwd).unwrap();
            let origin = repo.find_remote("origin").unwrap();
            let repo_url = origin.url().unwrap();
            let splitted_repo_url: Vec<_> = repo_url.split("github.com/").collect();
            let splitted_git_url: Vec<_> = splitted_repo_url[1].split(".git").collect();
            let github_repo_name = splitted_git_url[0];
            println!("full_name: {:#?}", github_repo_name);

            let resp = get_repository_info(github_repo_name).await?;
            println!("users: {:#?}", resp);

            repo.remote_set_url("upstream", &resp.parent.ssh_url)
                .unwrap();
        }
        None => println!("No subcommand_namend was used"),
        _ => unreachable!(),
    }

    Ok(())
}

async fn get_repository_info(repo_name: &str) -> Result<Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo_name);
    let response = client
        .get(&url)
        .header(USER_AGENT, "Piyopiyo")
        .send()
        .await?;
    println!("status: {:#?}", response.status());
    response.json::<Response>().await
}
