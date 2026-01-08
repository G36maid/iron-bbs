use russh::server::Handle;
use russh::ChannelId;
use std::io;
use tokio::sync::mpsc::{unbounded_channel, UnboundedSender};

pub struct TerminalHandle {
    sender: UnboundedSender<Vec<u8>>,
    sink: Vec<u8>,
}

impl TerminalHandle {
    pub async fn start(handle: Handle, channel_id: ChannelId) -> Self {
        let (sender, mut receiver) = unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(data) = receiver.recv().await {
                let result = handle.data(channel_id, data.into()).await;
                if result.is_err() {
                    tracing::error!("Failed to send data: {:?}", result);
                }
            }
        });
        Self {
            sender,
            sink: Vec::new(),
        }
    }
}

impl io::Write for TerminalHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sink.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = self.sender.send(self.sink.clone());
        if result.is_err() {
            return Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                result.unwrap_err(),
            ));
        }
        self.sink.clear();
        Ok(())
    }
}
