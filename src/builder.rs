use crate::repo::Repo;
use crate::types::*;
use url::Url;

/// Builder-pattern API for constructing a `Repo` with a fluent interface
#[derive(Debug, Clone)]
pub struct RepoBuilder {
    repo: Repo,
}

impl RepoBuilder {
    pub fn new(id: RepoId) -> Self {
        RepoBuilder {
            repo: Repo::new(id),
        }
    }

    pub fn from(existing: &Repo) -> Self {
        RepoBuilder {
            repo: existing.clone(),
        }
    }

    #[must_use]
    pub fn build(self) -> Repo {
        self.repo
    }

    pub fn name(mut self, v: RepoName) -> Self {
        self.repo.name = Some(v);
        self
    }

    pub fn baseurl(mut self, v: Url) -> Self {
        self.repo.baseurl.push(v);
        self
    }

    pub fn baseurls(mut self, v: Vec<Url>) -> Self {
        self.repo.baseurl = v;
        self
    }

    pub fn mirrorlist(mut self, v: Url) -> Self {
        self.repo.mirrorlist = Some(v);
        self
    }

    pub fn metalink(mut self, v: Url) -> Self {
        self.repo.metalink = Some(v);
        self
    }

    pub fn gpgkey(mut self, v: &str) -> Self {
        self.repo.gpgkey.push(v.to_string());
        self
    }

    pub fn gpgkeys(mut self, v: Vec<String>) -> Self {
        self.repo.gpgkey = v;
        self
    }

    pub fn enabled(mut self, v: DnfBool) -> Self {
        self.repo.enabled = Some(v);
        self
    }

    pub fn gpgcheck(mut self, v: DnfBool) -> Self {
        self.repo.gpgcheck = Some(v);
        self
    }

    pub fn repo_gpgcheck(mut self, v: DnfBool) -> Self {
        self.repo.repo_gpgcheck = Some(v);
        self
    }

    pub fn priority(mut self, v: Priority) -> Self {
        self.repo.priority = Some(v);
        self
    }

    pub fn cost(mut self, v: Cost) -> Self {
        self.repo.cost = Some(v);
        self
    }

    pub fn module_hotfixes(mut self, v: DnfBool) -> Self {
        self.repo.module_hotfixes = Some(v);
        self
    }

    pub fn metadata_type(mut self, v: RepoMetadataType) -> Self {
        self.repo.metadata_type = Some(v);
        self
    }

    pub fn mediaid(mut self, v: &str) -> Self {
        self.repo.mediaid = Some(v.to_string());
        self
    }

    pub fn excludepkgs(mut self, v: &str) -> Self {
        self.repo.excludepkgs.push(v.to_string());
        self
    }

    pub fn includepkgs(mut self, v: &str) -> Self {
        self.repo.includepkgs.push(v.to_string());
        self
    }

    pub fn skip_if_unavailable(mut self, v: DnfBool) -> Self {
        self.repo.skip_if_unavailable = Some(v);
        self
    }

    pub fn retries(mut self, v: Retries) -> Self {
        self.repo.retries = Some(v);
        self
    }

    pub fn timeout(mut self, v: TimeoutSeconds) -> Self {
        self.repo.timeout = Some(v);
        self
    }

    pub fn max_parallel_downloads(mut self, v: MaxParallelDownloads) -> Self {
        self.repo.max_parallel_downloads = Some(v);
        self
    }

    pub fn proxy(mut self, v: ProxySetting) -> Self {
        self.repo.proxy = v;
        self
    }

    pub fn username(mut self, v: Username) -> Self {
        self.repo.username = Some(v);
        self
    }

    pub fn password(mut self, v: Password) -> Self {
        self.repo.password = Some(v);
        self
    }

    pub fn extra(mut self, key: &str, value: &str) -> Self {
        self.repo
            .extras
            .entry(key.to_string())
            .or_default()
            .push(value.to_string());
        self
    }
}
