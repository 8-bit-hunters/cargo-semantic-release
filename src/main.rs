use git2::{Commit, Repository};
use std::env;

fn main() {
    let path = env::current_dir().expect("Failed to get current directory");
    println!("Current directory: {}", path.display());

    let git_repo = Repository::open(path).expect("Failed to open git repo");
    let mut revwalk = git_repo.revwalk().expect("Failed to revwalk");
    revwalk.push_head().expect("Failed to push head");

    let commits: Vec<Commit> = revwalk
        .filter_map(|object_id| object_id.ok())
        .filter_map(|valid_object_id| git_repo.find_commit(valid_object_id).ok())
        .collect();

    println!("Commits in the directory:");
    for commit in commits {
        println!(
            "\t{} - {}",
            commit.id(),
            commit.message().unwrap().trim_end()
        );
    }
}
