use jack::prelude as j;

pub struct Notifications;

impl j::NotificationHandler for Notifications {
    fn thread_init(&self, _: &j::Client) {
        println!("JACK: thread init");
    }

    fn shutdown(&mut self, status: j::ClientStatus, reason: &str) {
        println!("JACK: shutdown with status {:?} because \"{}\"",
                 status,
                 reason);
    }

    fn freewheel(&mut self, _: &j::Client, is_enabled: bool) {
        println!("JACK: freewheel mode is {}",
                 if is_enabled { "on" } else { "of" });
    }

    fn buffer_size(&mut self, _: &j::Client, sz: j::JackFrames) -> j::JackControl {
        println!("JACK: buffer size changed to {}", sz);
        j::JackControl::Continue
    }

    fn sample_rate(&mut self, _: &j::Client, srate: j::JackFrames) -> j::JackControl {
        println!("JACK: sample rate changed to {}", srate);
        j::JackControl::Continue
    }

    fn client_registration(&mut self, _: &j::Client, name: &str, is_reg: bool) {
        println!("JACK: {} client with name \"{}\"",
                 if is_reg { "registered" } else { "unregistered" },
                 name);
    }

    fn port_registration(&mut self, _: &j::Client, port_id: j::JackPortId, is_reg: bool) {
        println!("JACK: {} port with id {}",
                 if is_reg { "registered" } else { "unregistered" },
                 port_id);
    }

    fn port_rename(&mut self,
                   _: &j::Client,
                   port_id: j::JackPortId,
                   old_name: &str,
                   new_name: &str)
                   -> j::JackControl {
        println!("JACK: port with id {} renamed from {} to {}",
                 port_id,
                 old_name,
                 new_name);
        j::JackControl::Continue
    }

    fn ports_connected(&mut self,
                       _: &j::Client,
                       port_id_a: j::JackPortId,
                       port_id_b: j::JackPortId,
                       are_connected: bool) {
        println!("JACK: ports with id {} and {} are {}",
                 port_id_a,
                 port_id_b,
                 if are_connected {
                     "connected"
                 } else {
                     "disconnected"
                 });
    }

    fn graph_reorder(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: graph reordered");
        j::JackControl::Continue
    }

    fn xrun(&mut self, _: &j::Client) -> j::JackControl {
        println!("JACK: xrun occurred");
        j::JackControl::Continue
    }

    fn latency(&mut self, _: &j::Client, mode: j::LatencyType) {
        println!("JACK: {} latency has changed",
                 match mode {
                     j::LatencyType::Capture => "capture",
                     j::LatencyType::Playback => "playback",
                 });
    }
}