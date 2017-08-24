#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate jack;

extern crate osc;

use jack::prelude as j;
use chan_signal::Signal;
use std::sync::mpsc::channel;

use osc::graphics::*;
use osc::notifications::*;
use osc::parseopts::*;




fn main() {
    let o = get_options();


    // Signal gets a value when the OS sent a INT or TERM signal.
    let signal = chan_signal::notify(&[Signal::INT, Signal::TERM]);
    // When our work is complete, send a sentinel value on `sdone`.
    // let (sdone, rdone) = chan::sync(0);
    let (snd, rcv) = channel();

    // Create client
    let (client, _status) = j::Client::new("rscope", j::client_options::NO_START_SERVER)
        .unwrap();

    println!("Sample rate is {}", client.sample_rate());

    // Register ports. They will be used in a callback that will be
    // called when new data is available.
    let in_a = client.register_port("in_1", j::AudioInSpec::default()).unwrap();
    let in_b = client.register_port("in_2", j::AudioInSpec::default()).unwrap();
    let process_callback = move |_: &j::Client, ps: &j::ProcessScope| -> j::JackControl {
        let in_a_p : &[f32] = &j::AudioInPort::new(&in_a, ps);
        let in_b_p : &[f32] = &j::AudioInPort::new(&in_b, ps);
        let lines = in_a_p.iter().cloned().zip(in_b_p.iter().cloned()).collect();
        snd.send(lines);
        println!("send");
        j::JackControl::Continue
    };


    let process = j::ClosureProcessHandler::new(process_callback);

    // Activate the client, which starts the processing.
    let active_client = j::AsyncClient::new(client, Notifications, process).unwrap();

    ::std::thread::spawn(move || run_graphics(o, rcv));

    // Wait for a signal or for work to be done.
    chan_select! {
        signal.recv() -> signal => {}
    }
    active_client.deactivate().unwrap();
}
