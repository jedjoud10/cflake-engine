use hypo_others::SmartList;

use crate::{Element, Root};

// Testing
#[test]
pub fn test() {
    let mut root = Root {
        smart_element_list: SmartList::default(),
    };
    let element = Element::new(&mut root, &veclib::Vector2::ZERO, &veclib::Vector2::ONE);
    let mut element2 = Element::new(&mut root, &veclib::Vector2::ONE, &veclib::Vector2::ONE);
    Element::attach(&mut root, element, vec![element2]);
    println!("{:?}", root.
);
}