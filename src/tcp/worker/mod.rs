use std::ops::Deref;
use std::time::Duration;

use tokio::sync::watch::Receiver;
use tokio::time;

use crate::tcp::data::{Controller, ReceiveData};

pub mod receive_processor;

impl Controller {
    pub(crate) async fn process_receiver<F>(&self, mut receiver: Receiver<Option<ReceiveData>>, process: F)
        where
            F: Fn(&ReceiveData) -> (),
    {
        let mut intv = time::interval(Duration::from_millis(10));

        loop {
            if let Some(r) = receiver.borrow_and_update().deref() {
                process(r);
            }

            if receiver.changed().await.is_err() {
                break;
            }
            intv.tick().await;
        }
    }
}