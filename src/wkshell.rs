use wind_kvstore::modules::shell;


fn main() -> anyhow::Result<()> {
    let mut shell = shell::Shell::new();
    shell.run()
}
