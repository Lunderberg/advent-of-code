pub fn gcd(a: i32, b: i32) -> i32 {
    if b == 0 {
        a
    } else {
        gcd(b, a.rem_euclid(b))
    }
}

pub fn lcm(a: i32, b: i32) -> i32 {
    a * b / gcd(a, b)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gcd() {
        assert_eq!(gcd(10, 8), 2);
        assert_eq!(gcd(13, 17), 1);
        assert_eq!(gcd(13, -35), 1);
        assert_eq!(gcd(-4000, 35), 5);
    }

    #[test]
    fn test_lcm() {
        assert_eq!(lcm(10, 8), 40);
        assert_eq!(lcm(13, 17), 221);
        assert_eq!(lcm(13, -35), -455);
        assert_eq!(lcm(-4000, 35), -28000);
    }
}
