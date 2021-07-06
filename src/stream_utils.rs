use std::io::Read;

/**
 * @func read_all
 * Read all of the bytes out of a socket and return the result as a vector
 */
pub fn read_all<T>(stream: &mut T, chunk_size: usize) -> std::io::Result<Vec::<u8>>
where T: Read {
    let mut out_buf = Vec::<u8>::new();
    let mut bytes_read = chunk_size;

    while bytes_read == chunk_size {
        let mut buffer = vec![0; chunk_size];
        bytes_read = stream.read(&mut buffer)?;
        out_buf.append(&mut buffer);
    }
    Ok(out_buf)
}
