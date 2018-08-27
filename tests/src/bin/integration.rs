#![feature(integer_atomics)]
#![allow(unused)]

extern crate fantoccini;
extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate commandspec;
#[macro_use]
extern crate taken;
extern crate rand;
#[macro_use]
extern crate failure;
extern crate rustc_serialize;

use fantoccini::{
    Client,
    Locator,
    error,
};
// use futures::prelude::*;
use commandspec::*;
use failure::Error;
use futures::future::{
    ok,
    Future,
};
use rand::thread_rng;
use std::process::Stdio;
use std::sync::atomic::{
    AtomicU16,
    Ordering,
};
use std::sync::{
    Arc,
    Barrier,
};

static DRIVER_PORT_COUNTER: AtomicU16 = AtomicU16::new(4445);

fn sleep_ms(val: u64) {
    ::std::thread::sleep(::std::time::Duration::from_millis(val))
}

fn in_ci() -> bool {
    ::std::env::var("CI")
        .ok()
        .map(|x| x == "true")
        .unwrap_or(false)
}

fn main() {
    commandspec::forward_ctrlc();

    let test_id1 = format!("test{}", random_id());
    let test_id2 = test_id1.clone();

    let both_barrier = Arc::new(Barrier::new(2));
    let seq_barrier = Arc::new(Barrier::new(2));

    let j1 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || run(&test_id1, both_barrier, Some(seq_barrier))
    });
    let j2 = ::std::thread::spawn({
        take!(=both_barrier, =seq_barrier);
        move || {
            seq_barrier.wait();
            println!("ok...");
            run(&test_id2, both_barrier, None)
        }
    });

    let ret1 = j1.join().unwrap().expect("Program failed:");
    let ret2 = j2.join().unwrap().expect("Program failed:");

    assert!(ret1, "client 1 failed to see ghosts");
    assert!(ret2, "client 2 failed to see ghosts");

    eprintln!("test successful.");
}

fn random_id() -> String {
    let mut rng = thread_rng();
    return ::rand::seq::sample_iter(&mut rng, 0..26u8, 8)
        .unwrap()
        .into_iter()
        .map(|x| (b'a' + x) as char)
        .collect::<String>();
}

#[allow(unused)]
#[derive(Debug)]
enum Driver {
    Chrome,
    Gecko,
}


struct JsCode<'a> {
    client: &'a Client,
    value: String,
}

fn code<'a>(client: &'a Client) -> JsCode<'a> {
    JsCode {
        client: client,
        value: "".to_string(),
    }
}

impl<'a> JsCode<'a> {
    fn js(mut self, input: &str) -> JsCode<'a> {
        self.value.push_str(input);
        self
    }

    fn keypress(self, key: &str) -> JsCode<'a> {
        self.js(&format!(r#"
var event = new KeyboardEvent("keypress", {{
    bubbles: true,
    cancelable: true,
    charCode: {},
}});
document.dispatchEvent(event);
            "#, key))
    }

    fn mousedown(self, x: &str, y: &str) -> JsCode<'a> {
        self.js(&format!(r#"
var evt = new MouseEvent("mousedown", {{
    bubbles: true,
    cancelable: true,
    clientX: {},
    clientY: {},
}});
document.querySelector('.edit-text').dispatchEvent(evt);
            "#, x, y))
    }

    fn execute(self) -> impl Future<Item = ::rustc_serialize::json::Json, Error = error::CmdError> {
        self.client.execute(&self.value, vec![])
    }
}

fn run(
    test_id: &str,
    both_barrier: Arc<Barrier>,
    seq_barrier: Option<Arc<Barrier>>,
) -> Result<bool, Error> {
    // TODO accept port ID and alternative drivers.
    let port = DRIVER_PORT_COUNTER.fetch_add(1, Ordering::Relaxed);
    let driver = Driver::Gecko; // TODO do not hardcode this

    println!("---> Connecting to driver {:?} on port {:?}", driver, port);

    let mut cmd = match driver {
        Driver::Chrome => {
            let mut cmd = command!("chromedriver")?;
            cmd.arg(format!("--port={}", port)).arg(port.to_string());
            cmd
        }
        Driver::Gecko => {
            let mut cmd = command!("geckodriver")?;
            cmd.arg("-p").arg(port.to_string());
            cmd
        }
    };

    // Launch child.
    let _webdriver_guard = cmd
        // .stdout(Stdio::inherit())
        // .stderr(Stdio::inherit())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn_guard()?;

    // Wait for webdriver startup.
    ::std::thread::sleep(::std::time::Duration::from_millis(3_000));

    // Connect
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let c = Client::new(&format!("http://0.0.0.0:{}/", port), &core.handle());
    let c: Client = core.run(c).unwrap();

    println!("Connected...");

    let ret_value = {
        // we want to have a reference to c so we can use it in the and_thens below
        let c = &c;

        // now let's set up the sequence of steps we want the browser to take
        // first, go to the Wikipedia page for Foobar
        let f = c
            .goto(&format!("http://0.0.0.0:8000/{}", test_id))
            .and_then(move |_| c.current_url())
            .and_then(move |url| {
                println!("1");

                // Wait for page to load
                sleep_ms(2_000);

                // Align threads
                if let Some(seq_barrier) = seq_barrier {
                    seq_barrier.wait();
                }
                both_barrier.wait();
                println!("Barrier done, continuing...");

                // assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foobar");
                // click "Foo (disambiguation)"
                c.wait_for_find(Locator::Css(r#"div[data-tag="caret"]"#))
            })
            .and_then(|_| {
                sleep_ms(1_000);

                println!("2");
                code(c)
                    .js(r#"

let marker = document.querySelector('.edit-text div[data-tag=h1] span');
let clientX = marker.getBoundingClientRect().right;
let clientY = marker.getBoundingClientRect().top;


                "#)
                    .mousedown("clientX - 3", "clientY + 3")
                    .execute()
            })
            .and_then(|_| {
                sleep_ms(1_000);

                println!("2a");
                code(c)
                    //.keypress("35")
                    .keypress("0x1f47b")
                    .execute()
            })
            .and_then(|_| {
                // Enough time for both clients to sync up.
                ok(sleep_ms(4000))
            })
            .and_then(|_| {
                println!("3");

                code(c)
                    .js(r#"

let h1 = document.querySelector('.edit-text div[data-tag=h1]');
return h1.innerText;

                    "#)
                    .execute()
            });

        // and set the browser off to do those things
        core.run(f).unwrap()
    };

    // drop the client to delete the browser session
    if let Some(fin) = c.close() {
        // and wait for cleanup to finish
        core.run(fin).unwrap();
    }

    println!("4. OUT {:?}", ret_value);
    let h1_string = ret_value.as_string().unwrap();
    eprintln!("done: {:?}", h1_string);

    // drop(child);

    Ok(h1_string.ends_with("👻👻"))
}
