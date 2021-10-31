pub use nodejs;

#[cfg(test)]
mod tests {
    use super::nodejs;

    #[test]
    fn it_works() {
        nodejs::hello()
    }
}
