use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::net::{UdpSocket,SocketAddr};

pub struct CommsManager<TInput,TOutput> 
{
    in_tx: Sender<TInput>,
    in_rx: Receiver<TInput>,
    out_tx: Sender<TOutput>,
    out_rx: Receiver<TOutput>,
    jh: Vec<thread::JoinHandle<()>>
}


impl <TInput,TOutput> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + 'static,
        TOutput: std::marker::Send + 'static 
{
    pub fn swap_inputs(self: &mut Self, payload: TInput) -> TInput {
        // -- this signals the thread that we want to exchange buffers
        self.in_tx.send(payload).unwrap();
        // -- wait for the thread to return the other buffer to us
        // -- if the thread is doing something else, there might be a delay before this returns
        // -- TODO: minimize downtime, maybe send the signal early and use async/await to await the result later?
        self.in_rx.recv().unwrap()
    }

    pub fn swap_outputs(self: &mut Self, payload: TOutput) -> TOutput {
        self.out_tx.send(payload).unwrap();
        self.out_rx.recv().unwrap()
    }

    pub fn finish(self: &mut Self) {
        // TODO: signal threads that we want to quit

        for j in self.jh {
            j.join().unwrap();
        }
    }
}


pub fn start_udp<TInput,TOutput>(in_address: &String) -> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + 'static,
        TOutput: std::marker::Send + 'static 
{
    let add = in_address.clone();

    // -- channels to communicate with reciever thread
    let (tx_own, rx_ext): (Sender<TInput>, Receiver<TInput>) = mpsc::channel();
    let (tx_ext, rx_own): (Sender<TInput>, Receiver<TInput>) = mpsc::channel();

    // -- channels to communicate with transmitter thread
    let (tx_own_out, rx_ext_out): (Sender<TOutput>, Receiver<TOutput>) = mpsc::channel();
    let (tx_ext_out, rx_own_out): (Sender<TOutput>, Receiver<TOutput>) = mpsc::channel();

    UdpSocket::bind(in_address);

    // spawn a thread to handle incoming data
    let jh = thread::spawn(move || {
        //input_udp::input_handler(&add, tx_own, rx_own)
        
    });

    // spawn another thread to handle sending data
    let jh2 = thread::spawn(move || {
        
    });

    CommsManager {
        in_tx: tx_ext,
        in_rx: rx_ext,
        out_tx: tx_ext_out,
        out_rx: rx_ext_out,
        jh: vec![jh,jh2]
    }
}