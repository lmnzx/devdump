use rand::Rng;

pub fn generate_id(length: u32) -> String {
    let characters: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut rng = rand::thread_rng();

    let x = (0..length)
        .map(|_| characters[rng.gen_range(0..characters.len())])
        .collect::<String>();

    return x;
}
