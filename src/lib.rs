// MIT License

// Copyright (c) 2022 Dawid Kubiszewski (dawidkubiszewski@gmail.com)

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.


pub mod dkubiszewski {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
    };

    pub struct TcpEcho {
        chunk_size: usize,
        port: usize,
    }

    impl TcpEcho {
        pub fn new(port: usize, chunk_size: usize) -> Self {
            Self {
                port: port,
                chunk_size: chunk_size,
            }
        }

        pub fn serve(&self) {
            self.serve_with_peek(|_: &[u8]| {});
        }

        pub fn serve_with_peek(&self, fn_peek: fn(&[u8])) {
            println!("Starting...");
            let listener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).unwrap();
            println!("Started.");
            let mut connection_counter = 0;
            for stream in listener.incoming() {
                connection_counter += 1;
                // TODO: use logger
                println!("Open new connection: {}", connection_counter);
                let mut stream = stream.unwrap();
                let mut buffer = vec![0u8; self.chunk_size];
                thread::spawn(move || loop {
                    let mut write_size = 0;
                    let read_size = stream.read(&mut buffer).unwrap();
                    if read_size == 0 {
                        println!(
                            "No more data to read closing connection: {}",
                            connection_counter
                        );
                        break;
                    }
                    fn_peek(&buffer[..read_size]);

                    while write_size < read_size {
                        write_size += stream.write(&buffer[..read_size - write_size]).unwrap();
                    }
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpStream,
        thread,
    };

    use crate::dkubiszewski::TcpEcho;

    #[test]
    fn send_receive() {
        let sut = TcpEcho::new(5555, 1024);

        thread::spawn(move || {
            sut.serve();
        });

        let mut test_stream = TcpStream::connect("localhost:5555").unwrap();

        let test_data: [u8; 4] = [0x10, 0x9, 0x3, 0x1];
        assert_eq!(test_data.len(), test_stream.write(&test_data).unwrap());
        let mut result_data = [0; 4];
        assert_eq!(test_data.len(), test_stream.read(&mut result_data).unwrap());

        assert_eq!(result_data, test_data);
    }

    #[test]
    fn send_receive_multiple_chunks() {
        let sut = TcpEcho::new(5556, 1024);

        thread::spawn(move || {
            sut.serve();
        });

        let mut test_stream = TcpStream::connect("localhost:5556").unwrap();

        let test_data2 = [
            [0x10u8, 0x9, 0x3, 0x1],
            [0x9u8, 0x8, 0x2, 0x0],
            [0x11u8, 0xa, 0x4, 0x2],
        ];
        for test_data in test_data2 {
            assert_eq!(test_data.len(), test_stream.write(&test_data).unwrap());
            let mut result_data = vec![0u8; test_data.len()];
            assert_eq!(test_data.len(), test_stream.read(&mut result_data).unwrap());

            assert_eq!(result_data, test_data);
        }
    }

    #[test]
    fn send_receive_multiple_connections() {
        let sut = TcpEcho::new(5557, 1024);

        thread::spawn(move || {
            sut.serve();
        });

        let test_data2 = [
            [0x10u8, 0x9, 0x3, 0x1],
            [0x9u8, 0x8, 0x2, 0x0],
            [0x11u8, 0xa, 0x4, 0x2],
        ];

        for test_data in test_data2 {
            let mut test_stream = TcpStream::connect("localhost:5557").unwrap();

            assert_eq!(test_data.len(), test_stream.write(&test_data).unwrap());
            let mut result_data = [0; 4];
            assert_eq!(test_data.len(), test_stream.read(&mut result_data).unwrap());

            assert_eq!(result_data, test_data);
        }
    }
}
