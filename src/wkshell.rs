use wind_kvstore::types::shell;


fn main() -> anyhow::Result<()> {
    let mut shell = shell::Shell::new();
    shell.run()
}
