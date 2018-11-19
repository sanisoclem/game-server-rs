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
    fn encode(self: &Self, buf: &mut [u8]);
}

pub struct CommsManager<TInput,TOutput> 
{
    in_tx: Sender<Box<Vec<(SocketAddr,TInput)>>>,
    in_rx: Receiver<Box<Vec<(SocketAddr,TInput)>>>,
    in_ctx: channel::Sender<ControlSignal>,
    out_tx: Sender<Box<Vec<(SocketAddr,TOutput)>>>,
    out_rx: Receiver<Box<Vec<(SocketAddr,TOutput)>>>,
    out_ctx: channel::Sender<ControlSignal>,
    jh: Vec<thread::JoinHandle<()>>
}


impl <TInput,TOutput> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + 'static,
        TOutput: std::marker::Send + 'static 
{
    pub fn swap_inputs(self: &mut Self, payload: Box<Vec<(SocketAddr,TInput)>>) -> Box<Vec<(SocketAddr,TInput)>> {
        // -- signal the input thread that we want to swap
        self.in_ctx.send(ControlSignal::Swap).unwrap();
        // -- send the processed buffer
        self.in_tx.send(payload).unwrap();
        // -- wait for the thread to return the other buffer (has new inputs to process)
        self.in_rx.recv().unwrap()
    }

    pub fn swap_outputs(self: &mut Self, payload: Box<Vec<(SocketAddr,TOutput)>>) -> Box<Vec<(SocketAddr,TOutput)>> {
        self.out_ctx.send(ControlSignal::Swap).unwrap();
        self.out_tx.send(payload).unwrap();
        self.out_rx.recv().unwrap()
    }

    pub fn finalize(self: &mut Self) {
        // signal threads that we want to quit
        self.in_ctx.send(ControlSignal::Terminate).unwrap();
        self.out_ctx.send(ControlSignal::Terminate).unwrap();
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
    let (in_tx_int, in_rx_ext) = mpsc::channel::<Box<Vec<(SocketAddr,TInput)>>>();
    let (in_tx_ext, in_rx_int) = mpsc::channel::<Box<Vec<(SocketAddr,TInput)>>>();
    let (in_tx_ctl, in_rx_ctl) = channel::channel::<ControlSignal>();

    // -- channels to communicate with transmitter thread
    let (out_tx_int, out_rx_ext)  = mpsc::channel::<Box<Vec<(SocketAddr,TOutput)>>>();
    let (out_tx_ext, out_rx_int) = mpsc::channel::<Box<Vec<(SocketAddr,TOutput)>>>();
    let(out_tx_ctl, out_rx_ctl) = channel::channel::<ControlSignal>();
    
    
    let address = in_address.parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(&address).unwrap();
    let socket_clone = socket.try_clone().unwrap();
    
    // spawn a thread to handle incoming data
    let jh = thread::Builder::new().name("input".to_owned()).spawn(move || {
        start_udp_input(socket, in_tx_int, in_rx_int, in_rx_ctl);
    }).unwrap();

    // spawn another thread to handle sending data
    let jh2 = thread::Builder::new().name("output".to_owned()).spawn(move || {
        start_udp_output(socket_clone,out_tx_int, out_rx_int, out_rx_ctl);
    }).unwrap();

    CommsManager {
        in_tx: in_tx_ext,
        in_rx: in_rx_ext,
        in_ctx: in_tx_ctl,
        out_tx: out_tx_ext,
        out_rx: out_rx_ext,
        out_ctx: out_tx_ctl,
        jh: vec![jh,jh2]
    }
}

fn start_udp_output<TOutput>(
    socket: UdpSocket,
    out_tx: mpsc::Sender<Box<Vec<(SocketAddr,TOutput)>>>,
    out_rx: mpsc::Receiver<Box<Vec<(SocketAddr,TOutput)>>>,
    out_ctx: channel::Receiver<ControlSignal>)
    where TOutput : std::marker::Send + OutputDatagram + 'static
{
    const OUTPUT: Token = Token(2);
    const OUTPUT_CTL: Token = Token(3);

    let mut buf = [0;512]; 
    let mut exit_requested = false;
    let mut output_buffer: Box<Vec<(SocketAddr,TOutput)>> = Box::new(Vec::new());
    let poll = mio::Poll::new().unwrap();

    poll.register(&socket, OUTPUT, Ready::writable(), PollOpt::edge()).unwrap();
    poll.register(&out_ctx, OUTPUT_CTL, Ready::readable(), PollOpt::level()).unwrap();
    
    // Create storage for events
    let mut events = Events::with_capacity(1024);

    println!("Spawned output thread");

    while !exit_requested {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                OUTPUT => {
                    while let Some((addr,dg)) = output_buffer.pop() {
                        dg.encode(&mut buf);
                        match socket.send_to(&buf, &addr) {
                            Ok(_) => {}
                            Err(s) => {
                                println!("Error sending {0}", s);
                            }
                        };
                    }
                }
                OUTPUT_CTL => {
                    match out_ctx.try_recv() {
                        Ok(ControlSignal::Swap) => {
                            // -- swap buffers
                            out_tx.send(output_buffer).unwrap();
                            output_buffer = out_rx.recv().unwrap();
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

    println!("Exiting output thread");
}

fn start_udp_input<TInput>(
    socket: UdpSocket,
    in_tx: mpsc::Sender<Box<Vec<(SocketAddr,TInput)>>>, 
    in_rx: mpsc::Receiver<Box<Vec<(SocketAddr,TInput)>>>,
    in_ctx: channel::Receiver<ControlSignal>)
    where TInput: std::marker::Send + InputDatagram + 'static, 
{
    const INPUT: Token = Token(0);
    const INPUT_CTL: Token = Token(1);

    let mut buf = [0;512]; 
    let mut exit_requested = false;
    let mut input_buffer: Box<Vec<(SocketAddr,TInput)>> = Box::new(Vec::new());
    let poll = mio::Poll::new().unwrap();

    poll.register(&socket, INPUT, Ready::readable(), PollOpt::level()).unwrap();
    poll.register(&in_ctx, INPUT_CTL, Ready::readable(), PollOpt::level()).unwrap();
    
    // Create storage for events
    let mut events = Events::with_capacity(1024);

    println!("Spawned input thread");

    while !exit_requested {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                INPUT => {
                    let (_,addr) =  socket.recv_from(&mut buf).unwrap();
                    input_buffer.push((addr, TInput::decode(&buf)));
                }
                INPUT_CTL => {
                    match in_ctx.try_recv() {
                        Ok(ControlSignal::Swap) => {
                            // -- swap buffers
                            in_tx.send(input_buffer).unwrap();
                            input_buffer = in_rx.recv().unwrap();
                            input_buffer.clear();

                            //println!("swapped buffers");
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