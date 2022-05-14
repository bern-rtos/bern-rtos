use crate::sync::ipc_channel::IpcChannel;

pub fn channel() -> IpcChannel {
    IpcChannel::new()
}
