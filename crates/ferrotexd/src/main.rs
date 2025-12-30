use ferrotexd::run_server;

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    run_server(stdin, stdout).await;
}
