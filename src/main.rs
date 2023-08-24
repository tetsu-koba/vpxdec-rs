//extern crate env_libvpx_sys;

use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};

// These are stubs for IVF and VP8Dec in Zig
// mod ivf {
//     pub struct IvfReader {
//         // stub
//     }

//     pub struct IvfFrameHeader {
//         // stub
//     }

//     impl IvfReader {
//         pub fn init(_file: File) -> Result<Self, ()> {
//             // stub
//         }

//         pub fn read_ivf_frame_header(&self, _header: &mut IvfFrameHeader) -> Result<(), ()> {
//             // stub
//         }

//         // ... other necessary methods
//     }
// }
use libivf_rs as ivf;

mod vp8dec {
    use std::error::Error;

    pub struct VP8Dec {
        // stub
        rawvideo: Vec<u8>,
    }

    impl VP8Dec {
        pub fn init(_fourcc: &[u8; 4]) -> Result<VP8Dec, Box<dyn Error>> {
            // stub
            Ok(VP8Dec {
                rawvideo: vec![0u8; 10],
            })
        }

        pub fn decode(&mut self, _frame_buffer: &[u8]) -> Result<&[u8], Box<dyn Error>> {
            // stub
            Ok(&self.rawvideo)
        }

        // ... other necessary methods
    }
}

fn decode(
    input_file: &str,
    output_file: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let ivf_file = File::open(input_file)?;
    let mut outfile = OpenOptions::new()
        .write(true)
        .create(true)
        .open(output_file)?;

    let mut reader = ivf::IvfReader::init(ivf_file)?;
    // ... checks for reader.header
    if reader.header.width != width as _ || reader.header.height != height as _ {
        return Err("Video size mismatch".into());
    }

    let mut frame_index = 0;
    let bufsize = 64 * 1024;
    let mut frame_buffer = vec![0u8; bufsize];
    let fourcc = reader.header.fourcc;
    let mut vp8dec = vp8dec::VP8Dec::init(&fourcc)?;

    loop {
        match reader.read_ivf_frame_header() {
            Ok(frame_header) => {
                let len: usize = frame_header.frame_size as _;
                match reader.read_frame(&mut frame_buffer[..len]) {
                    Ok(_) => {}
                    Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        eprintln!("Error: {e:?}");
                        break;
                    }
                }
                match vp8dec.decode(&frame_buffer[..len]) {
                    Ok(raw_video) => match outfile.write_all(raw_video) {
                        Ok(_) => {}
                        Err(ref e) if e.kind() == ErrorKind::BrokenPipe => break,
                        Err(e) => {
                            eprintln!("Error: {e:?}");
                            break;
                        }
                    },
                    Err(e) => {
                        eprintln!("Error: {e:?}");
                        break;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
        frame_index += 1;
    }

    _ = frame_index;
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} input_file output_file width height", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let width: u32 = args[3].parse().expect("Invalid width");
    let height: u32 = args[4].parse().expect("Invalid height");

    match decode(input_file, output_file, width, height) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("Error occurred");
            std::process::exit(1);
        }
    }
}
