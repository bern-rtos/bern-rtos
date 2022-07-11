use crate::sync::ipc::channel::IpcChannel;

pub fn channel() -> IpcChannel {
    IpcChannel::new()
}
