#[cfg(test)]
pub mod tests {
    use rand::thread_rng;
    use rand::{distributions::Alphanumeric, Rng};

    pub fn gen_alpha_char(chars: usize) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(chars).map(char::from).collect()
    }
    pub fn gen_char(options: &[u8]) -> String {
        let mut rng = thread_rng();
        (options[rng.gen_range(0..options.len())] as char).to_string()
    }
    pub fn gen_bool() -> bool {
        thread_rng().gen_range(0..1) != 0
    }
    pub fn gen_usize_from_range(from: usize, to: usize) -> usize {
        thread_rng().gen_range(from..to)
    }
}
