// The smart list system
pub struct SmartList<T> {
    pub elements: Vec<Option<T>>
}

// All the smart list logic
impl<T> SmartList<T> where T: Clone {
    // Calculate the next valid ID from the actual elements
    pub fn get_next_valid_id(&self) -> u16 {
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
            .unwrap_or(self.elements.len()) as u16;
    }
    // Add an element, find a free spot first though
    pub fn add_element(&mut self, element: T) -> u16 {
        // Get the id of the elements inside the temp vector (Local ID)
        let id = self.get_next_valid_id();
        // Update
        if id < self.elements.len() as u16 {
            // Turn the none into a valid element
            self.elements[id as usize] = Some(element.clone());
        } else {
            // Add this to the elements
            self.elements.push(Some(element.clone()));
        }
        id
    }
    // Remove an element from this SmartList
    pub fn remove_element(&mut self, element_id: &u16) -> Option<Option<T>> {
        // Remove the element
        return Some(self.elements.remove(*element_id as usize));
    }
    // Get a mutable reference to a stored element
    pub fn get_element_mut(&mut self, element_id: &u16) -> Option<&mut T> {
        if *element_id < self.elements.len() as u16 {
            let element = self.elements.get_mut(*element_id as usize).unwrap().as_mut().unwrap();
            return Some(element);
        } else {
            return None;
        }
    }
    // Get an entity using it's entity id
    pub fn get_element(&self, element_id: &u16) -> Option<&T> {
        if *element_id < self.elements.len() as u16 {
            let element = self.elements.get(*element_id as usize).unwrap().as_ref().unwrap();
            return Some(element);
        } else {
            return None;
        }
    }
}