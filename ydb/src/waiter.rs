use crate::YdbResult;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::watch;

#[async_trait::async_trait]
pub trait Waiter: Send + Sync {
    async fn wait(&self) -> YdbResult<()>;
}

pub(crate) struct WaiterImpl {
    received_succesfull: AtomicBool,
    sender: watch::Sender<YdbResult<bool>>,
    receiver: watch::Receiver<YdbResult<bool>>,
}

impl WaiterImpl {
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(YdbResult::Ok(false));
        WaiterImpl {
            received_succesfull: AtomicBool::new(false),
            sender,
            receiver,
        }
    }

    pub fn set_received(&self, res: YdbResult<()>) {
        // fast return if received already
        if self.received_succesfull.load(Ordering::Relaxed) {
            return;
        }

        let success = res.is_ok();
        let send_val = match res {
            Ok(()) => Ok(true),
            Err(err) => Err(err),
        };
        let _ = self.sender.send(send_val);
        if success {
            self.received_succesfull.store(true, Ordering::Relaxed)
        }
    }
}

#[async_trait::async_trait]
impl Waiter for WaiterImpl {
    async fn wait(&self) -> YdbResult<()> {
        if self.received_succesfull.load(Ordering::Relaxed) {
            return Ok(());
        };

        let mut receiver = self.receiver.clone();
        loop {
            if receiver.borrow_and_update().clone()? {
                return Ok(());
            }
            receiver.changed().await?;
        }
    }
}

#[async_trait::async_trait]
impl Waiter for Arc<WaiterImpl> {
    async fn wait(&self) -> YdbResult<()> {
        return self.as_ref().wait().await;
    }
}
