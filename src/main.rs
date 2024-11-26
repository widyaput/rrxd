use std::env;
use std::fs;
use std::io::Write;
use std::io::{BufReader, Read, Seek, SeekFrom};

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut group: u8 = 2;
    let use_little_endian = args.contains(&String::from("-e"));
    if use_little_endian {
        group = 4;
    }
    if let Some(pos) = args.iter().position(|x| x == "-g") {
        if let Some(group_count) = args.get(pos + 1) {
            if let Ok(count) = group_count.parse::<u8>() {
                if count == 0 || 16 % count == 0 {
                    group = count
                }
            }
        }
    }

    let mut limit: Option<usize> = None;
    if let Some(pos) = args.iter().position(|x| x == "-l") {
        if let Some(limit_count) = args.get(pos + 1) {
            if let Ok(count) = limit_count.parse::<usize>() {
                limit = Some(count)
            }
        }
    }

    let mut column: usize = 16;
    if let Some(pos) = args.iter().position(|x| x == "-c") {
        if let Some(column_count) = args.get(pos + 1) {
            if let Ok(count) = column_count.parse::<usize>() {
                column = count
            }
        }
    }
    let mut seek_pos: usize = 0;
    if let Some(pos) = args.iter().position(|x| x == "-s") {
        if let Some(seek_count) = args.get(pos + 1) {
            if let Ok(count) = seek_count.parse::<usize>() {
                seek_pos = count;
            }
        }
    }

    let input = args.last();
    match input {
        Some(filename) => output_lines(
            preprocess_seek(fs::File::open(filename).unwrap(), seek_pos),
            use_little_endian,
            group,
            limit,
            column,
            seek_pos,
        ),
        None => output_lines(
            preprocess(std::io::stdin(), seek_pos),
            use_little_endian,
            group,
            limit,
            column,
            seek_pos,
        ),
    };
}

fn preprocess<R: Read>(reader: R, seek_pos: usize) -> BufReader<R> {
    let mut buffer = BufReader::new(reader);
    if seek_pos > 0 {
        let mut discard = vec![0; seek_pos];
        buffer.read_exact(&mut discard).unwrap();
    }
    buffer
}

fn preprocess_seek<R: Read + Seek>(reader: R, seek_pos: usize) -> BufReader<R> {
    let mut buffer = BufReader::new(reader);
    buffer.seek(SeekFrom::Start(seek_pos as u64)).unwrap();
    buffer
}

fn output_lines<R: Read>(
    reader: R,
    use_little_endian: bool,
    group: u8,
    limit: Option<usize>,
    column: usize,
    seek_pos: usize,
) {
    let mut buffer = BufReader::new(reader);
    let mut raw_buffer: Vec<u8> = vec![0; column];
    let mut prefix = seek_pos;
    let mut local_group = group;
    if group == 0 {
        local_group = column as u8;
    }
    loop {
        if let Some(limit_count) = limit {
            if prefix >= limit_count + seek_pos {
                break;
            }
        }
        match buffer.read(&mut raw_buffer[..]) {
            Ok(0) => break,
            Ok(n) => {
                print!("{:08x}: ", prefix);
                let mut i = 0;
                'hex_loop: while i < column {
                    if use_little_endian {
                        for j in (0..local_group).rev() {
                            if let Some(limit_count) = limit {
                                if prefix + i + usize::from(local_group) - 1 - usize::from(j)
                                    >= limit_count + seek_pos
                                {
                                    break 'hex_loop;
                                }
                            }
                            if i + usize::from(j) < n {
                                print!("{:02x}", raw_buffer[i + usize::from(j)]);
                            } else {
                                print!("00")
                            }
                        }
                    } else {
                        for j in 0..local_group {
                            if let Some(limit_count) = limit {
                                if prefix + i + usize::from(j) >= limit_count + seek_pos {
                                    break 'hex_loop;
                                }
                            }
                            if i + usize::from(j) < n {
                                print!("{:02x}", raw_buffer[i + usize::from(j)]);
                            } else {
                                print!("00")
                            }
                        }
                    }
                    print!(" ");
                    i += usize::from(local_group);
                }
                print!(" ");
                'text_loop: for (i, el) in raw_buffer.iter().enumerate() {
                    if let Some(limit_count) = limit {
                        if prefix + i >= limit_count - 1 + seek_pos {
                            break 'text_loop;
                        }
                    }
                    if (32..=126).contains(el) {
                        print!("{}", *el as char)
                    } else {
                        print!(".")
                    }
                }
                prefix += column;
                if std::io::stdout().write(b"\n").is_err() {
                    break;
                }
            }
            _ => break,
        }
    }
}
