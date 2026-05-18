use crate::types::*;
use camino::Utf8PathBuf;
use indexmap::IndexMap;
use url::Url;

/// A fully typed [repo-id] section
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    // ===== repo-only =====
    pub id: RepoId,
    pub name: Option<RepoName>,
    pub baseurl: Vec<Url>,
    pub mirrorlist: Option<Url>,
    pub metalink: Option<Url>,
    pub gpgkey: Vec<String>,
    pub enabled: Option<DnfBool>,
    pub priority: Option<Priority>,
    pub cost: Option<Cost>,
    pub module_hotfixes: Option<DnfBool>,
    pub metadata_type: Option<RepoMetadataType>,
    pub mediaid: Option<String>,
    pub enabled_metadata: Vec<String>,

    // ===== shared =====
    pub excludepkgs: Vec<String>,
    pub includepkgs: Vec<String>,
    pub gpgcheck: Option<DnfBool>,
    pub repo_gpgcheck: Option<DnfBool>,
    pub localpkg_gpgcheck: Option<DnfBool>,
    pub skip_if_unavailable: Option<DnfBool>,
    pub deltarpm: Option<DnfBool>,
    pub deltarpm_percentage: Option<DeltaRpmPercentage>,
    pub enablegroups: Option<DnfBool>,
    pub fastestmirror: Option<DnfBool>,
    pub countme: Option<DnfBool>,
    pub bandwidth: Option<StorageSize>,
    pub throttle: Option<Throttle>,
    pub minrate: Option<StorageSize>,
    pub retries: Option<Retries>,
    pub timeout: Option<TimeoutSeconds>,
    pub max_parallel_downloads: Option<MaxParallelDownloads>,
    pub metadata_expire: Option<MetadataExpire>,
    pub ip_resolve: Option<IpResolve>,
    pub sslverify: Option<DnfBool>,
    pub sslverifystatus: Option<DnfBool>,
    pub sslcacert: Option<Utf8PathBuf>,
    pub sslclientcert: Option<Utf8PathBuf>,
    pub sslclientkey: Option<Utf8PathBuf>,
    pub proxy: ProxySetting,
    pub proxy_username: Option<ProxyUsername>,
    pub proxy_password: Option<ProxyPassword>,
    pub proxy_auth_method: Option<ProxyAuthMethod>,
    pub proxy_sslverify: Option<DnfBool>,
    pub proxy_sslcacert: Option<Utf8PathBuf>,
    pub proxy_sslclientcert: Option<Utf8PathBuf>,
    pub proxy_sslclientkey: Option<Utf8PathBuf>,
    pub username: Option<Username>,
    pub password: Option<Password>,
    pub user_agent: Option<UserAgent>,

    // ===== unknown =====
    pub extras: IndexMap<String, Vec<String>>,
}

impl Repo {
    /// Create a new Repo with only an ID, all fields to defaults
    pub fn new(id: RepoId) -> Self {
        Repo {
            id,
            name: None,
            baseurl: Vec::new(),
            mirrorlist: None,
            metalink: None,
            gpgkey: Vec::new(),
            enabled: None,
            priority: None,
            cost: None,
            module_hotfixes: None,
            metadata_type: None,
            mediaid: None,
            enabled_metadata: Vec::new(),
            excludepkgs: Vec::new(),
            includepkgs: Vec::new(),
            gpgcheck: None,
            repo_gpgcheck: None,
            localpkg_gpgcheck: None,
            skip_if_unavailable: None,
            deltarpm: None,
            deltarpm_percentage: None,
            enablegroups: None,
            fastestmirror: None,
            countme: None,
            bandwidth: None,
            throttle: None,
            minrate: None,
            retries: None,
            timeout: None,
            max_parallel_downloads: None,
            metadata_expire: None,
            ip_resolve: None,
            sslverify: None,
            sslverifystatus: None,
            sslcacert: None,
            sslclientcert: None,
            sslclientkey: None,
            proxy: ProxySetting::Unset,
            proxy_username: None,
            proxy_password: None,
            proxy_auth_method: None,
            proxy_sslverify: None,
            proxy_sslcacert: None,
            proxy_sslclientcert: None,
            proxy_sslclientkey: None,
            username: None,
            password: None,
            user_agent: None,
            extras: IndexMap::new(),
        }
    }

    /// Determine the URL source for this repo.
    pub fn url_source(&self) -> Option<UrlSource> {
        if !self.baseurl.is_empty() {
            Some(UrlSource::BaseUrl(self.baseurl.clone()))
        } else if let Some(ref url) = self.mirrorlist {
            Some(UrlSource::MirrorList(url.clone()))
        } else {
            self.metalink
                .as_ref()
                .map(|url| UrlSource::Metalink(url.clone()))
        }
    }
}
