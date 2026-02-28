fn main() {
    println!("hello");
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_math() {
        assert_eq!(1 + 1, 2);
    }
}
