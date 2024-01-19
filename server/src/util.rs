use rand::Rng;

pub fn generate_id() -> String {
    let characters: Vec<char> = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789"
        .chars()
        .collect();
    let mut rng = rand::thread_rng();

    (0..6)
        .map(|_| characters[rng.gen_range(0..characters.len())])
        .collect::<String>()
}
