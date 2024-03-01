use poem::{
    handler,
    listener::{Listener, TcpListener},
    Result, Route, Server,
};

#[handler]
fn hello() -> String {
    "Hello from poem!\n".to_string()
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = Route::new().at("/", poem::get(hello));
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "poem=debug");
    }
    tracing_subscriber::fmt::init();

    // To test port assignment, run two instances of this example at once.
    //
    // For ports <1024, running with administrator priveledges would be needed
    // on Unix. For port 0, the OS would assign a port and we'd need to find out
    // what that port's number is.
    let (min_port, max_port) = (8080, 8085);
    // Using 127.0.0.1 instead of 0.0.0.0 for security; a local server should
    // not, generally, be visible from the network.
    let hostname = "127.0.0.1";
    let mut port = min_port;
    let mut error = None;
    let acceptor = loop {
        if port > max_port {
            return Err(error.unwrap());
        }
        let listener = TcpListener::bind(format!("{hostname}:{port}"));
        match listener.into_acceptor().await {
            Ok(a) => break a,
            Err(err) => error = Some(err),
        };
        // Most likely, another application is bound to this port.
        eprintln!("Couldn't bind to port {port}.");
        port += 1;
    };
    // Now that the acceptor exists, the browser should be able to connect
    eprintln!("Listening at {hostname}:{port}.");

    tokio::task::spawn_blocking(move || {
        // We use spawn_blocking in case `open::that` blocks. This happens
        // occasionally, for example when launching a fresh instance of
        // `firefox` on Linux. Note that killing the server process would also
        // kill the browser in this case.
        //
        // Alternatively, you can use `open::that_detached`, but that would
        // report success even if the browser exited with a non-0 error code.
        let http_address = format!("http://{hostname}:{port}/");
        eprintln!("Trying to launch a browser at {http_address}...");
        match open::that(&http_address) {
            Ok(()) => eprintln!("Browser launched successfully."),
            Err(err) => eprintln!("Failed to launch a browser: {err}"),
        }
    });

    Server::new_with_acceptor(acceptor).run(app).await?;
    Ok(())
}
