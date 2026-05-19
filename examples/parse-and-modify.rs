use dnf_repofile::*;

fn main() -> Result<()> {
    let input = "\
[epel]
name=Extra Packages for Enterprise Linux $releasever - $basearch
baseurl=https://download.example.com/pub/epel/$releasever/$basearch/
enabled=1
gpgcheck=1
gpgkey=https://download.example.com/pub/epel/RPM-GPG-KEY
";
    let mut rf = RepoFile::parse(input)?;

    println!("Parsed {} repos:", rf.len());
    for (id, block) in &rf {
        let enabled = block.data.enabled == Some(DnfBool::True);
        let name = block
            .data
            .name
            .as_deref()
            .map(|s| s.as_str())
            .unwrap_or("(unnamed)");
        println!("  [{}] {} (enabled: {})", id, name, enabled);
    }

    // Modify a repo
    let epel_id = RepoId::try_new("epel").unwrap();
    if let Some(block) = rf.get_mut(&epel_id) {
        block.data.gpgcheck = Some(DnfBool::True);
    }

    // Validate each repo manually
    for (_id, block) in &rf {
        let report = block.data.validate();
        if report.is_ok() {
            println!("  [{}] validation: PASS", _id);
        } else {
            for err in &report.errors {
                eprintln!("  [{}] Validation error: {}", _id, err.message);
            }
        }
    }

    Ok(())
}
