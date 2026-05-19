use dnf_repofile::*;

fn main() -> Result<()> {
    // In real usage: ReposDir::load("/etc/yum.repos.d")
    let mut rf = RepoFile::new();

    // Add repos programmatically
    let repo = RepoBuilder::new(RepoId::try_new("custom").unwrap())
        .name(RepoName::try_new("Custom Repository").unwrap())
        .baseurl("https://custom.example.com/".parse().unwrap())
        .gpgcheck(DnfBool::yes())
        .enabled(DnfBool::yes())
        .priority(Priority::try_new(50).unwrap())
        .build();

    rf.add(repo)?;

    // Render to string
    println!("{}", rf);

    // Diff
    let mut other = RepoFile::new();
    other.add(
        RepoBuilder::new(RepoId::try_new("custom").unwrap())
            .name(RepoName::try_new("Updated").unwrap())
            .baseurl("https://updated.example.com/".parse().unwrap())
            .build(),
    )?;

    let diff = diff_files(&rf, &other);
    println!("Changes detected: {}", diff.has_changes);

    Ok(())
}
