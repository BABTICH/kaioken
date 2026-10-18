use kaioken::config::Config;

#[test]
fn test_config_defaults_and_roundtrip() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = Config::default_config();
    config.save(temp_dir.path()).unwrap();

    let config_file = temp_dir.path().join(".agent").join("config.toml");
    assert!(config_file.exists());

    let loaded = Config::load(temp_dir.path()).unwrap();
    assert_eq!(loaded.project.name, "my-project");
    assert_eq!(loaded.memory.database, ".agent/memory.sqlite");
    assert_eq!(loaded.docs.output, ".agent-docs");
    assert_eq!(loaded.llm.provider, "openrouter");
}

#[test]
fn test_config_load_nonexistent_fails() {
    let temp_dir = tempfile::tempdir().unwrap();
    let result = Config::load(temp_dir.path());
    assert!(result.is_err());
}
