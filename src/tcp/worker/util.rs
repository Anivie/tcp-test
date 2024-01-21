use std::ops::Deref;
use std::time::Duration;

use tokio::sync::watch::Receiver;
use tokio::time;

use crate::tcp::packet::data::{Controller, ReceiveData, SpacilProcessor};

macro_rules! processor {
    ($self:ident, $receiver:expr, $spacil:expr, $f:expr) => {
        $self.process_receiver($receiver, $spacil, $f).await
    };
}

impl Controller {
    pub(crate) async fn process_receiver<F>(&self, mut receiver: Receiver<Option<ReceiveData>>, spacil: SpacilProcessor, process: F)
        where
            F: Fn(&ReceiveData) -> (),
    {
        let mut intv = time::interval(Duration::from_millis(10));

        loop {
            if let Some(r) = receiver.borrow_and_update().deref() {
                if spacil != *self.spacil.read().deref() {
                    continue;
                }

                process(r);
            }

            if receiver.changed().await.is_err() {
                break;
            }
            intv.tick().await;
        }
    }
}