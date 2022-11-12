#[cfg(test)]
mod bitset {
    use crate::{BitSet};    

    #[test]
    fn bitset() {
        let mut bitset = BitSet::new();
        assert!(!bitset.get(0));
        bitset.set(0);
        assert!(bitset.get(0));
        assert_eq!(bitset.chunks().len(), 1);
        bitset.set(64);
        bitset.set(65);
        assert_eq!(bitset.chunks().len(), 2);
        assert!(bitset.get(64));
        assert!(bitset.get(65));

        bitset.remove(0);
        assert!(!bitset.get(0));
        assert!(bitset.get(64));
        assert!(bitset.get(65));
    }

    #[test]
    fn counting() {
        let mut bitset = BitSet::new();
        bitset.set(0);
        bitset.set(10);

        assert_eq!(bitset.count_ones(), 2);
        assert_eq!(bitset.find_one_from(0), Some(0));
        assert_eq!(bitset.find_one_from(1), Some(10));
        
        bitset.set(4096);
        assert_eq!(bitset.find_one_from(10), Some(10));
        assert_eq!(bitset.find_one_from(11), Some(4096));
    }
}

#[cfg(test)]
mod hibitset {
    use crate::{HiBitSet};

    #[test]
    fn hibitset() {
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