use std::process::Command;
use async_std::prelude::*;

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let mut app = tide::new();


    let cmd = async {
        if Command::new("firefox").arg("http://localhost:8080").spawn().is_err() {
        panic!("unable to launch firefox");}
    };

    app.at("/").get(|_| async { 
        Ok("Hello, world!") 
    });

    let (_, res) = cmd.join(app.listen("127.0.0.1:8080")).await;
    res?;

    Ok(())
}
