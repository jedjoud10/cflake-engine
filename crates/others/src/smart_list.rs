// The smart list system
#[derive(Debug, Clone)]
pub struct SmartList<T>
where
    T: Sized,
{
    pub elements: Vec<Option<T>>,
    pub size_in_bytes: usize,
}

impl<T> Default for SmartList<T> {
    fn default() -> Self {
        Self {
            elements: Vec::new(),
            size_in_bytes: 0,
        }
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
            .position(|(_i, e)| {
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
        self.size_in_bytes += std::mem::size_of_val(&element);
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
        self.elements.push(None);
        let element = self.elements.swap_remove(element_id);
        self.size_in_bytes -= std::mem::size_of_val(element.as_ref()?);
        return element;
    }
    // Get a mutable reference to a stored element
    pub fn get_element_mut(&mut self, element_id: usize) -> Option<Option<&mut T>> {
        if element_id < self.elements.len() {
            let element = self.elements.get_mut(element_id)?.as_mut();
            return Some(element);
        } else {
            return None;
        }
    }
    // Get an entity using it's entity id
    pub fn get_element(&self, element_id: usize) -> Option<Option<&T>> {
        if element_id < self.elements.len() {
            let element = self.elements.get(element_id)?.as_ref();
            return Some(element);
        } else {
            return None;
        }
    }
    // Get all the valid elements
    pub fn get_valids(&self) -> Vec<&T> {
        let x: Vec<&T> = self.elements.iter().filter_map(|x| x.as_ref()).collect();
        return x;
    }
    // Remove all the elements
    pub fn remove_all(&mut self) {
        self.elements.clear();
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
