use libivf_rs as ivf;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::process::ExitCode;
mod vpxdec;

fn decode(
    input_file: &str,
    output_file: &str,
    width: u32,
    height: u32,
) -> Result<(), Box<dyn Error>> {
    let mut outfile = File::create(output_file)?;
    let mut reader = ivf::IvfReader::init(File::open(input_file)?)?;
    if reader.header.width != width as _ || reader.header.height != height as _ {
        return Err("Video size mismatch".into());
    }

    let mut frame_buffer = Vec::<u8>::new();
    let mut vpxdec = vpxdec::VpxDec::init(&reader.header.fourcc)?;

    let mut frame_index = 0;
    loop {
        if let Err(e) = do_frame(&mut reader, &mut vpxdec, &mut frame_buffer, &mut outfile) {
            if let Some(io_err) = e.downcast_ref::<io::Error>() {
                match io_err.kind() {
                    ErrorKind::UnexpectedEof | ErrorKind::BrokenPipe => break,
                    _ => {}
                }
            }
            return Err(e);
        }
        frame_index += 1;
    }

    _ = frame_index;
    Ok(())
}

fn do_frame(
    reader: &mut ivf::IvfReader,
    vpxdec: &mut vpxdec::VpxDec,
    frame_buffer: &mut Vec<u8>,
    outfile: &mut File,
) -> Result<(), Box<dyn Error>> {
    let frame_header = reader.read_ivf_frame_header()?;
    let frame_size: usize = frame_header.frame_size as _;
    if frame_buffer.len() < frame_size {
        frame_buffer.resize(frame_size, 0);
    }
    reader.read_frame(&mut frame_buffer[..frame_size])?;
    vpxdec.decode(&frame_buffer[..frame_size])?;
    while let Some(img) = vpxdec.get_frame() {
        img.write_all(outfile)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} input_file output_file width height", args[0]);
        return ExitCode::FAILURE;
    }

    let input_file = &args[1];
    let output_file = &args[2];
    let width: u32 = args[3].parse().expect("Invalid width");
    let height: u32 = args[4].parse().expect("Invalid height");

    match decode(input_file, output_file, width, height) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e:?}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}
