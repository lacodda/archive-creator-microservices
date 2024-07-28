use rand::{distributions::Alphanumeric, Rng};

pub fn generate_random_password() -> String {
    rand::thread_rng().sample_iter(&Alphanumeric).take(20).map(char::from).collect()
}
