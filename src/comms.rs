use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::net::{SocketAddr,ToSocketAddrs};
use mio_extras::channel;
use mio::net::UdpSocket;
use mio::{Token,Ready,PollOpt,Evented,Events};

pub enum ControlSignal {
    Terminate,
    Swap,
}

pub trait InputDatagram {
    fn decode(buf: &[u8]) -> Self;
}

pub trait OutputDatagram {
    fn encode(s: &Self, buf: &mut [u8]);
}

pub struct CommsManager<TInput,TOutput> 
{
    in_tx: Sender<Box<Vec<TInput>>>,
    in_rx: Receiver<Box<Vec<TInput>>>,
    in_ctx: channel::Sender<ControlSignal>,
    out_tx: Option<TOutput>,
    // out_rx: Receiver<TOutput>,
    // out_ctx: channel::Sender<ControlSignal>,
    jh: Vec<thread::JoinHandle<()>>
}


impl <TInput,TOutput> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + 'static,
        TOutput: std::marker::Send + 'static 
{
    pub fn swap_inputs(self: &mut Self, payload: Box<Vec<TInput>>) -> Box<Vec<TInput>> {
        // -- signal the input thread that we want to swap
        self.in_ctx.send(ControlSignal::Swap);
        // -- send the processed buffer
        self.in_tx.send(payload).unwrap();
        // -- wait for the thread to return the other buffer (has new inputs to process)
        self.in_rx.recv().unwrap()
    }

    // pub fn swap_outputs(self: &mut Self, payload: TOutput) -> TOutput {
    //     self.out_tx.send(payload).unwrap();
    //     self.out_rx.recv().unwrap()
    // }

    pub fn finalize(self: &mut Self) {
        // TODO: signal threads that we want to quit
        while let Some(i) = self.jh.pop() {
            i.join().unwrap();
        }
    }
}

pub fn start_udp<TInput,TOutput>(in_address: &String) -> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + InputDatagram + 'static,
        TOutput: std::marker::Send + OutputDatagram + 'static 
{

    // -- channels to communicate with reciever thread
    let (in_tx_int, in_rx_ext): (Sender<Box<Vec<TInput>>>, Receiver<Box<Vec<TInput>>>) = mpsc::channel();
    let (in_tx_ext, in_rx_int): (Sender<Box<Vec<TInput>>>, Receiver<Box<Vec<TInput>>>) = mpsc::channel();
    let (in_tx_ctl, in_rx_ctl): (channel::Sender<ControlSignal>, channel::Receiver<ControlSignal>) = channel::channel();

    // -- channels to communicate with transmitter thread
    
    
    let address = in_address.parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(&address).unwrap();
    //let thread_socket = socket.try_clone().unwrap();
    

    // spawn a thread to handle incoming data
    let jh = thread::spawn(move || {
        start_udp_input(socket, in_tx_int, in_rx_int, in_rx_ctl);
    });

    // spawn another thread to handle sending data
    // let jh2 = thread::spawn(move || {
    //     let buf = [0; 512];
    //     thread_socket.send(&buf);
    // });

    CommsManager {
        in_tx: in_tx_ext,
        in_rx: in_rx_ext,
        in_ctx: in_tx_ctl,
        out_tx: None,
        jh: vec![jh]
    }
}

fn start_udp_input<TInput>(
    socket: UdpSocket,
    in_tx: mpsc::Sender<Box<Vec<TInput>>>, 
    in_rx: mpsc::Receiver<Box<Vec<TInput>>>,
    in_ctx: channel::Receiver<ControlSignal>)
    where TInput: std::marker::Send + InputDatagram + 'static, 
{
    const INPUT: Token = Token(0);
    const INPUT_CTL: Token = Token(1);

    let mut buf = [0;512]; 
    let mut exit_requested = false;
    let mut inputBuffer: Box<Vec<TInput>> = Box::new(Vec::new());
    let poll = mio::Poll::new().unwrap();

    socket.register(&poll, INPUT, Ready::readable(), PollOpt::edge()).unwrap();
    in_ctx.register(&poll, INPUT_CTL, Ready::readable(), PollOpt::edge()).unwrap();
    
    // Create storage for events
    let mut events = Events::with_capacity(1024);

    println!("Spawned input thread");

    while !exit_requested {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                INPUT => {
                    socket.recv(&mut buf).unwrap();
                    inputBuffer.push(TInput::decode(&buf));
                }
                INPUT_CTL => {
                    match in_ctx.try_recv() {
                        Ok(ControlSignal::Swap) => {
                            // -- swap buffers
                            in_tx.send(inputBuffer).unwrap();
                            inputBuffer = in_rx.recv().unwrap();
                            inputBuffer.clear();

                            println!("swapped buffers");
                        },
                        Ok(ControlSignal::Terminate) => {
                            exit_requested = true;
                        },
                        _ => {
                            println!("Error receiving control signal");
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }

    println!("Exiting input thread");
}