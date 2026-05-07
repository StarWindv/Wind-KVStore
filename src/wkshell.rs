use wind_kvstore::shell;


fn main() -> anyhow::Result<()> {
    let mut shell = shell::Shell::new();
    shell.run()
}
