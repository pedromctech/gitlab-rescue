use std::io::Read;
use std::io::BufReader;

pub fn print_buffer(stdout: impl Read) {
    use std::io::BufRead;
    BufReader::new(stdout)
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
}
