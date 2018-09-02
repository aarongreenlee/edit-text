use crate::{
    Client,
    ClientImpl,
};

use extern::{
    crossbeam_channel::Sender,
    edit_common::commands::*,
    failure::Error,
};

#[cfg(not(target_arch = "wasm32"))]
pub struct ProxyClient {
    pub state: Client,
    pub tx_client: Sender<FrontendCommand>,
    pub tx_sync: Sender<ServerCommand>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ClientImpl for ProxyClient {
    fn state(&mut self) -> &mut Client {
        &mut self.state
    }

    fn send_client(&self, req: &FrontendCommand) -> Result<(), Error> {
        log_wasm!(SendClient(req.clone()));
        self.tx_client.send(req.clone())?;
        Ok(())
    }

    fn send_sync(&self, req: ServerCommand) -> Result<(), Error> {
        log_wasm!(SendSync(req.clone()));
        self.tx_sync.send(req)?;
        Ok(())
    }
}

// macro_rules! spawn_monkey_task {
//     ($alive:expr, $monkey:expr, $tx:expr, $wait_params:expr, $task:expr) => {{
//         let tx = $tx.clone();
//         let alive = $alive.clone();
//         let monkey = $monkey.clone();
//         thread::spawn::<_, Result<(), Error>>(move || {
//             let mut rng = rand::thread_rng();
//             while alive.load(Ordering::Relaxed) {
//                 thread::sleep(Duration::from_millis(
//                     rng.gen_range($wait_params.0, $wait_params.1),
//                 ));
//                 if monkey.load(Ordering::Relaxed) {
//                     tx.send(Task::ControllerCommand($task))?;
//                 }
//             }
//             Ok(())
//         })
//     }};
// }
