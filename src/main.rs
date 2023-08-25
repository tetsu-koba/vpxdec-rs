use libivf_rs as ivf;
use std::env;
use std::error::Error;
use std::fs::{File, OpenOptions};
use std::io::{ErrorKind, Write};
mod vpxdec;

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
    let mut vpxdec = vpxdec::VpxDec::init(&fourcc)?;

    loop {
        let frame_header = match reader.read_ivf_frame_header() {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        };
        let len: usize = frame_header.frame_size as _;
        match reader.read_frame(&mut frame_buffer[..len]) {
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
        let raw_video = match vpxdec.decode(&frame_buffer[..len]) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        };
        match outfile.write_all(raw_video) {
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::BrokenPipe => break,
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
