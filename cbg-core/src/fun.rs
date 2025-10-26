//! fun stuff (or misc i just didn't know where to put them)
//! currently houses pi and ipv4 randomizer
use core::f64::consts::PI;
use rand::Rng;

pub fn pi(rng: &mut impl Rng) -> String {
    let mut pi_string = format!("{:.15}", core::f64::consts::PI); // get Pi to 15 decimal places

    // VERY SMALL CHANCE to mess up a digit
    if rng.random_bool(PI.fract()) {
        let digits: Vec<char> = pi_string.chars().collect();

        // pick a random index after the decimal point (skip '3' and '.')
        let idx = rng.random_range(2..digits.len());
        let new_digit = rng.random_range(0..10).to_string().chars().next().unwrap();

        let mut new_pi_string = digits.clone();
        new_pi_string[idx] = new_digit;
        pi_string = new_pi_string.iter().collect();
    }

    pi_string
}

pub fn get_random_ipv4(rng: &mut impl Rng) -> String {
    let octet1: u8 = rng.random_range(0..=255);
    let octet2: u8 = rng.random_range(0..=255);
    let octet3: u8 = rng.random_range(0..=255);
    let octet4: u8 = rng.random_range(0..=255);

    format!("{}.{}.{}.{}", octet1, octet2, octet3, octet4)
}

#[cfg(test)]
mod test {
    use rand::{SeedableRng, rngs::StdRng};

    use super::*;

    #[test]
    fn test_pi() {
        let seed: u64 = 12345;
        let mut rng = StdRng::seed_from_u64(seed);
        let pi = pi(&mut rng)
            .parse::<f64>()
            .expect("HEY!! pi() returned non f64 value!");
        assert_eq!(pi, PI);
    }

    // #[test]
    // fn is_random_ipv4_vaild() {
    // let mut rng = StdRng::from_os_rng();
    // let ip = get_random_ipv4(&mut rng);
    // }
}
