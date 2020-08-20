
use anyhow::Result;
use tide::Request;

use futures::stream::StreamExt;
use futures::sink::SinkExt;

use tide::Body;
use http_types::ReadWrite;
use async_std::prelude::*;

#[derive(Clone)]
struct State {}

async fn async_main() -> Result<()> {
    
    let state = State {};
    let mut app = tide::with_state(state);
    
    app.at("/a").get(|_req: Request<State>| async move {
        Ok("a")
    });
    app.at("/u").all(|_req: Request<State>| async move {
        use tide::Response;
        let mut res = Response::new(200);
        
        let body = Body::io(|mut io: Box<dyn ReadWrite>| async move {
            let mut b = Vec::new();
            b.resize(4, 0);
            let x = io.read_exact(&mut b).await.unwrap();
            println!("{:?} {:?}", x, b);
            
            
            let _x = io.write_all(b"HTTP/1.1 200 Ok\n\n").await.unwrap();
            
            let x = io.write_all(&b).await.unwrap();
            io.flush().await.unwrap();
            println!("{:?}", x);
        });
        res.set_body(body);
        
        Ok(res)
    });
    // curl 'http://localhost:8080/u2' --data pyfg --request POST --no-buffer -i
    app.at("/u2").all(|mut req: Request<State>| async move {
        use tide::Response;
        let mut res = Response::new(200);
        
        let body = Body::io(|mut io: Box<dyn ReadWrite>| async move {
            
            let mut b = Vec::new();
            let x = req.take_body().read_to_end(&mut b).await.unwrap();
            println!("{:?} {:?}", x, b);
            
            let x = io.write_all(&b).await.unwrap();
            io.flush().await.unwrap();
            println!("{:?}", x);
        });
        res.set_body(body);
        
        Ok(res)
    });
    
    app.at("/ws").get(|req: Request<State>| async move {
        tide::websocket::upgrade(req, |_req, handle| async {
            let mut ws = handle.into_inner();
            
            loop {
                let msg = ws.next().await.unwrap().unwrap();
                dbg!(&msg);
                ws.send(msg).await.unwrap();
            }
        })
    });
    
    app.listen("0.0.0.0:8080").await?;
    
    Ok(())
}

fn main() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    
    futures::executor::block_on(async_main())?;
    
    Ok(())
}
