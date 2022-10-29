#[cfg(test)]
mod tests {
    use crate::HiBitSet;

    #[test]
    fn test() {
        let mut hibitset = HiBitSet::new();
        hibitset.set(0);
        assert_eq!(hibitset.get(0), true);
        assert_eq!(hibitset.get(1), false);
        assert_eq!(hibitset.get(0), true);
        hibitset.remove(0);
        assert_eq!(hibitset.get(0), false);

        hibitset.set(4095);
        assert_eq!(hibitset.get(4095), true);
    }
}
