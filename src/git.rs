use anyhow::Result;
use git2::{Repository, Signature};
use std::path::Path;

pub struct GitManager {
    repo: Repository,
}

impl GitManager {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = match Repository::open(path.as_ref()) {
            Ok(repo) => repo,
            Err(_) => Repository::init(path.as_ref())?,
        };
        Ok(Self { repo })
    }

    pub fn commit(&self, message: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all(["."].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        let signature = Signature::now("GithubDB", "githubdb@example.com")?;
        let parent_commit = match self.repo.head() {
            Ok(head) => Some(head.peel_to_commit()?),
            Err(_) => None,
        };

        let parents = parent_commit.as_ref().map(|c| vec![c]).unwrap_or_default();
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            parents.as_slice(),
        )?;

        Ok(())
    }
}
