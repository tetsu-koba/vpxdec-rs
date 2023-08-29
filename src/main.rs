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
    if reader.header.width != width as _ || reader.header.height != height as _ {
        return Err("Video size mismatch".into());
    }

    let mut frame_index = 0;
    let mut frame_buffer = Vec::<u8>::new();
    let fourcc = reader.header.fourcc;
    let mut vpxdec = vpxdec::VpxDec::init(&fourcc)?;

    'outer: loop {
        let frame_header = match reader.read_ivf_frame_header() {
            Ok(f) => f,
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        };
        let frame_size: usize = frame_header.frame_size as _;
        if frame_buffer.len() < frame_size {
            frame_buffer.resize(frame_size, 0);
        }
        match reader.read_frame(&mut frame_buffer[..frame_size]) {
            Ok(_) => {}
            Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => break,
            Err(e) => {
                eprintln!("Error: {e:?}");
                break;
            }
        }
        if let Err(e) = vpxdec.decode(&frame_buffer[..frame_size]) {
            eprintln!("Error: {e:?}");
            break;
        }
        loop {
            let img = vpxdec.get_frame();
            if img == 0 as _ {
                break;
            }
            unsafe {
                let img = *img;
                let mut ptr = img.planes[0];
                for _ in 0..img.d_h {
                    match outfile.write_all(std::slice::from_raw_parts(ptr, img.d_w as _)) {
                        Ok(_) => {}
                        Err(ref e) if e.kind() == ErrorKind::BrokenPipe => break 'outer,
                        Err(e) => {
                            eprintln!("Error: {e:?}");
                            break 'outer;
                        }
                    }
                    ptr = ptr.add(img.stride[0] as _);
                }
                for i in 1..3 {
                    let mut ptr = img.planes[i];
                    for _ in 0..img.d_h / 2 {
                        match outfile.write_all(std::slice::from_raw_parts(ptr, (img.d_w / 2) as _))
                        {
                            Ok(_) => {}
                            Err(ref e) if e.kind() == ErrorKind::BrokenPipe => break 'outer,
                            Err(e) => {
                                eprintln!("Error: {e:?}");
                                break 'outer;
                            }
                        }
                        ptr = ptr.add(img.stride[i] as _);
                    }
                }
            }
            frame_index += 1;
        }
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
