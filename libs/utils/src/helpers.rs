use rand::Rng;

pub fn generate_numeric_code(length: usize) -> String {
    assert!(length > 0);

    let mut rng = rand::rng();
    (0..length)
        .map(|_| rng.random_range(0..10).to_string())
        .collect()
}