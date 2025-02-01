use git2::Repository;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <repo_path>", args[0]);
        std::process::exit(1);
    }
    let repo_path = &args[1];
    
    let repo = Repository::open(repo_path).expect("Failed to open repository");
    let mut revwalk = repo.revwalk().expect("Failed to create revwalk");
    revwalk.set_sorting(git2::Sort::TIME | git2::Sort::REVERSE).expect("Failed to set sorting");
    
    for branch in repo.branches(None).expect("Failed to get branches") {
        if let Ok((branch, _)) = branch {
            if let Some(target) = branch.get().target() {
                revwalk.push(target).expect("Failed to push branch commit");
            }
        }
    }

    let mut contributions: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    let mut first_commit_date: Option<DateTime<Utc>> = None;
    let mut last_commit_date: Option<DateTime<Utc>> = None;

    for oid in revwalk {
        let oid = oid.expect("Failed to get commit ID");
        let commit = repo.find_commit(oid).expect("Failed to find commit");
        let author = commit.author();
        let name = author.name().unwrap_or("Unknown").to_string();
        let time = commit.time().seconds();
        
        let datetime = DateTime::<Utc>::from_timestamp(time, 0).expect("Invalid timestamp");
        let date_str = datetime.format("%Y-%m-%d").to_string();
        
        contributions.entry(name).or_default().insert(date_str);

        if first_commit_date.is_none() || datetime < first_commit_date.unwrap() {
            first_commit_date = Some(datetime);
        }
        if last_commit_date.is_none() || datetime > last_commit_date.unwrap() {
            last_commit_date = Some(datetime);
        }
    }
    
    let mut sorted_contributors: Vec<_> = contributions.iter().collect();
    sorted_contributors.sort_by_key(|(_, days)| days.len());
    sorted_contributors.reverse();
    
    let total_dev_days: usize = contributions.values().map(|days| days.len()).sum();
    
    println!("Developer Contribution Days:");
    for (developer, days) in sorted_contributors {
        println!("{}: {} days", developer, days.len());
    }
    println!("\nTotal Developer Contribution Days: {}", total_dev_days);

    if let (Some(first), Some(last)) = (first_commit_date, last_commit_date) {
        println!("\nCommit History Range: {} to {}", first.format("%Y-%m-%d"), last.format("%Y-%m-%d"));
    }
}
