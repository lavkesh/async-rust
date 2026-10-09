use std::io::{self, Read, Write};
use std::net::TcpStream;

use crate::ffi::{EV_EOF, EV_ERROR, EV_FILTER_READ, kevent};
use crate::poll::{Interest, Poll};

mod ffi;
mod poll;

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
         Host: localhost\r\n\
         Connection: close\r\n\
         \r\n"
    )
}

fn handle_events(
    poll: &Poll,
    events: &[kevent],
    streams: &mut [TcpStream],
) -> io::Result<usize> {
    let mut handled = 0;

    for event in events {
        let index = event.udata as usize;

        if event.flags & EV_ERROR != 0 {
            let err = io::Error::from_raw_os_error(event.data as i32);
            eprintln!("event {index} error: {err}");
            continue;
        }

        if event.filter != EV_FILTER_READ {
            continue;
        }

        let mut data = vec![0u8; 4096];
        let mut txt = String::new();

        // Edge-triggered (EV_CLEAR): must read until WouldBlock or EOF.
        loop {
            match streams[index].read(&mut data) {
                Ok(0) => {
                    handled += 1;
                    poll.registry().deregister(&streams[index], Interest::READABLE)?;
                    break;
                }
                Ok(n) => {
                    txt.push_str(&String::from_utf8_lossy(&data[..n]));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // Drained for now. If EV_EOF is set the peer is done,
                    // but the next read returns Ok(0), so we loop until then.
                    if event.flags & EV_EOF != 0 {
                        continue;
                    }
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => break,
                Err(e) => return Err(e),
            }
        }

        if !txt.is_empty() {
            println!("RECEIVED: {index}\n{txt}\n------\n");
        }
    }

    Ok(handled)
}

fn main() -> io::Result<()> {
    let poll = Poll::new()?;
    let n_events = 5;
    let mut streams: Vec<TcpStream> = vec![];
    let addr = "localhost:8080";

    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_req(&url_path);

        let mut stream = TcpStream::connect(addr)?;
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;
        stream.write_all(request.as_bytes())?;

        poll.registry().register(&stream, i, Interest::READABLE)?;
        streams.push(stream);
    }

    let mut handled_events = 0;
    while handled_events < n_events {
        let mut events: Vec<kevent> = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;

        if events.is_empty() {
            println!("TIMEOUT (OR SPURIOUS EVENT NOTIFICATION)");
            continue;
        }

        handled_events += handle_events(&poll, &events, &mut streams)?;
    }

    println!("FINISHED");
    Ok(())
}