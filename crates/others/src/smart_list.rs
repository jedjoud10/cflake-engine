// The smart list system
#[derive(Debug, Clone)]
pub struct SmartList<T> {
    pub elements: Vec<Option<T>>,
}

impl<T> Default for SmartList<T> {
    fn default() -> Self {
        Self { elements: Vec::new() }
    }
}

// All the smart list logic
impl<T> SmartList<T> {
    // Calculate the next valid ID from the actual elements
    pub fn get_next_valid_id(&self) -> usize {
        // Calculate the next valid free ID
        return self
            .elements
            .iter()
            .enumerate()
            .position(|(i, e)| {
                match e {
                    // We found a free spot
                    Some(_) => false,
                    None => true,
                }
            })
            .unwrap_or(self.elements.len());
    }
    // Add an element, find a free spot first though
    pub fn add_element(&mut self, element: T) -> usize {
        // Get the id of the elements inside the temp vector (Local ID)
        let id = self.get_next_valid_id();
        // Update
        if id < self.elements.len() {
            // Turn the none into a valid element
            self.elements[id as usize] = Some(element);
        } else {
            // Add this to the elements
            self.elements.push(Some(element));
        }
        id
    }
    // Remove an element from this SmartList
    pub fn remove_element(&mut self, element_id: usize) -> Option<T> {
        // Remove the element
        let element = std::mem::replace(&mut self.elements[element_id], None);
        return element;
    }
    // Get a mutable reference to a stored element
    pub fn get_element_mut(&mut self, element_id: usize) -> Option<&mut T> {
        if element_id < self.elements.len() {
            let element = self.elements.get_mut(element_id).unwrap().as_mut().unwrap();
            return Some(element);
        } else {
            return None;
        }
    }
    // Get an entity using it's entity id
    pub fn get_element(&self, element_id: usize) -> Option<&T> {
        if element_id < self.elements.len() {
            let element = self.elements.get(element_id as usize).unwrap().as_ref().unwrap();
            return Some(element);
        } else {
            return None;
        }
    }
    // Count how many valid elements we have inside the smart list
    pub fn count_valid(&self) -> usize {
        let c = self.elements.iter().filter(|x| x.is_some()).count();
        return c;
    }
    // Count how many invalid elements we have inside the smart list
    pub fn count_invalid(&self) -> usize {
        let c = self.elements.iter().filter(|x| x.is_none()).count();
        return c;
    }
}
