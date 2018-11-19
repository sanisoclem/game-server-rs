use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::net::{SocketAddr};
use mio_extras::channel;
use mio::net::UdpSocket;
use mio::{Token,Ready,PollOpt,Events};

pub enum ControlSignal {
    Terminate,
    SwapInput,
    SwapOutput
}

pub trait InputDatagram {
    fn deserialize(buf: &[u8]) -> Self;
}

pub trait OutputDatagram {
    fn encode(self: &Self, buf: &mut [u8]);
}

pub struct CommsManager<TInput,TOutput> 
{
    in_tx: Sender<Box<Vec<(SocketAddr,TInput)>>>,
    in_rx: Receiver<Box<Vec<(SocketAddr,TInput)>>>,
    ctx: channel::Sender<ControlSignal>,
    out_tx: Sender<Box<Vec<(SocketAddr,TOutput)>>>,
    out_rx: Receiver<Box<Vec<(SocketAddr,TOutput)>>>,
    jh: Vec<thread::JoinHandle<()>>
}


impl <TInput,TOutput> CommsManager<TInput,TOutput> 
    where 
        TInput: std::marker::Send + 'static,
        TOutput: std::marker::Send + 'static 
{
    pub fn swap_inputs(self: &mut Self, payload: Box<Vec<(SocketAddr,TInput)>>) -> Box<Vec<(SocketAddr,TInput)>> {
        // -- signal the input thread that we want to swap
        self.ctx.send(ControlSignal::SwapInput).unwrap();
        // -- send the processed buffer
        self.in_tx.send(payload).unwrap();
        // -- wait for the thread to return the other buffer (has new inputs to process)
        self.in_rx.recv().unwrap()
    }

    pub fn swap_outputs(self: &mut Self, payload: Box<Vec<(SocketAddr,TOutput)>>) -> Box<Vec<(SocketAddr,TOutput)>> {
        self.ctx.send(ControlSignal::SwapOutput).unwrap();
        self.out_tx.send(payload).unwrap();
        self.out_rx.recv().unwrap()
    }

    pub fn finalize(self: &mut Self) {
        // signal threads that we want to quit
        self.ctx.send(ControlSignal::Terminate).unwrap();
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
    // -- control channel
    let (tx_ctx, rx_ctx) = channel::channel::<ControlSignal>();

    // -- channels to communicate with reciever thread
    let (in_tx_int, in_rx_ext) = mpsc::channel::<Box<Vec<(SocketAddr,TInput)>>>();
    let (in_tx_ext, in_rx_int) = mpsc::channel::<Box<Vec<(SocketAddr,TInput)>>>();
    

    // -- channels to communicate with transmitter thread
    let (out_tx_int, out_rx_ext)  = mpsc::channel::<Box<Vec<(SocketAddr,TOutput)>>>();
    let (out_tx_ext, out_rx_int) = mpsc::channel::<Box<Vec<(SocketAddr,TOutput)>>>();
    let(out_tx_ctl, out_rx_ctl) = channel::channel::<ControlSignal>();
    
    
    let address = in_address.parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(&address).unwrap();
    let socket_clone = socket.try_clone().unwrap();
    
    // spawn a thread to handle incoming data
    let jh = thread::Builder::new().name("input".to_owned()).spawn(move || {
        start_udp_input(socket, in_tx_int, in_rx_int, out_tx_int, out_rx_int, rx_ctx);
    }).unwrap();


    CommsManager {
        in_tx: in_tx_ext,
        in_rx: in_rx_ext,
        out_tx: out_tx_ext,
        out_rx: out_rx_ext,
        ctx: tx_ctx,
        jh: vec![jh]
    }
}

fn start_udp_input<TInput,TOutput>(
    socket: UdpSocket,
    in_tx: mpsc::Sender<Box<Vec<(SocketAddr,TInput)>>>, 
    in_rx: mpsc::Receiver<Box<Vec<(SocketAddr,TInput)>>>,
    out_tx: mpsc::Sender<Box<Vec<(SocketAddr,TOutput)>>>,
    out_rx: mpsc::Receiver<Box<Vec<(SocketAddr,TOutput)>>>,
    ctx: channel::Receiver<ControlSignal>)
    where 
        TInput: std::marker::Send + InputDatagram + 'static, 
        TOutput: std::marker::Send + OutputDatagram + 'static, 
{
    const CTL: Token = Token(0);
    const SOCKET_IO: Token = Token(1);

    let mut buf = [0;512]; 
    let mut exit_requested = false;
    let mut input_buffer: Box<Vec<(SocketAddr,TInput)>> = Box::new(Vec::new());
    let mut output_buffer: Box<Vec<(SocketAddr,TOutput)>> = Box::new(Vec::new());
    let poll = mio::Poll::new().unwrap();

    poll.register(&ctx, CTL, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&socket, SOCKET_IO, Ready::readable() | Ready::writable(), PollOpt::edge()).unwrap();
    
    
    // Create storage for events
    let mut events = Events::with_capacity(1024);

    println!("Spawned io thread");

    while !exit_requested {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match (event.token(),event.readiness()) {
                (CTL,_) => {
                    match ctx.try_recv() {
                        Ok(ControlSignal::SwapInput) => {
                            // -- swap buffers
                            in_tx.send(input_buffer).unwrap();
                            input_buffer = in_rx.recv().unwrap();
                            input_buffer.clear();
                        },
                        Ok(ControlSignal::SwapOutput) => {
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
                },
                (SOCKET_IO,readiness) if readiness.is_writable()  => {
                    if let Some((addr,dg)) = output_buffer.pop() {
                        dg.encode(&mut buf);
                        match socket.send_to(&buf, &addr) {
                            Ok(_) => {}
                            Err(s) => {
                                println!("Error sending {0}", s);
                            }
                        };
                    }
                },
                (SOCKET_IO,readiness) if readiness.is_readable()  => {
                    let (_,addr) =  socket.recv_from(&mut buf).unwrap();
                    input_buffer.push((addr, TInput::deserialize(&buf)));
                },
                _ => unreachable!(),
            }
        }
    }

    println!("Exiting IO thread");
}