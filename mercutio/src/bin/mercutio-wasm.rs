extern crate mercutio;
extern crate serde_json;
extern crate ws;
extern crate structopt;
#[macro_use]
extern crate structopt_derive;

use structopt::StructOpt;
use std::thread;
use mercutio::wasm::start_websocket_server;
use std::time::Duration;
use mercutio::wasm::NativeCommand;

#[derive(StructOpt, Debug)]
#[structopt(name = "mercutio-wasm", about = "An example of StructOpt usage.")]
struct Opt {
    #[structopt(long="monkies", help = "Monkey count")]
    monkies: Option<usize>,

    #[structopt(long="port", help = "Port", default_value = "3011")]
    port: u16,
}


pub fn main() {
    println!("started \"wasm\" server");


    let opt = Opt::from_args();
    let port = opt.port;
    let monkies = opt.monkies;

    if monkies.is_some() {
        virtual_monkeys();
    }

    start_websocket_server(port);
}

fn virtual_monkeys() {
    println!("(!) virtual monkeys enabled");

    let opt = Opt::from_args();
    let port = opt.port;
    let monkies = opt.monkies.unwrap();

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(1000));

        for key in 0..monkies {
            thread::spawn(move || {
                let url = format!("ws://127.0.0.1:{}/{}", port, ('a' as u8 + key as u8) as char);
                println!("Connecting to {:?}", url);

                ws::connect(url.as_str(), move |out| {
                    thread::sleep(Duration::from_millis(1000 + ((key as u64) * 400)));
                    
                    // Start monkey
                    let command = NativeCommand::Monkey(true);
                    let json = serde_json::to_string(&command).unwrap();
                    out.send(json.as_str()).unwrap();

                    move |msg: ws::Message| {
                        // println!("wasm got a packet from sync '{}'. ", msg);

                        Ok(())
                    }
                }).unwrap();
            });
        }
    });
}