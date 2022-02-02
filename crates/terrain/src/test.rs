#[cfg(test)]
mod test {
    #[test]
    pub fn test() {
        dbg!(crate::flatten_custom(veclib::vec3(0, 0, 0), 2));
        dbg!(crate::flatten_custom(veclib::vec3(1, 0, 0), 2));
        dbg!(crate::flatten_custom(veclib::vec3(0, 1, 0), 2));
        dbg!(crate::flatten_custom(veclib::vec3(0, 0, 1), 2));
    }
}