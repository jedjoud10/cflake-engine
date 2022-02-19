use super::Edit;
use crate::ChunkCoords;
use math::octrees::Octree;

// An editing manager that contains all the world edits
pub struct EditingManager {
    // Collection of total edits
    edits: Vec<Edit>,
}

impl EditingManager {
    // Add a new edit
    pub fn edit(&mut self, edit: Edit) {
        self.edits.push(edit);
    }

    // Using an octree, check what chunks need to be edited
    pub fn fetch_updates(&mut self, octree: &Octree) -> Vec<ChunkCoords> {
        // Get the nodes
        let shapes = self.edits.iter().map(|edit| edit.shape.clone()).collect::<Vec<_>>();
        let nodes = math::intersection::shapes_octree(&shapes, octree);
        // Get the chunks coordiantes
        nodes.into_iter().map(|node| ChunkCoords::new(node)).collect::<Vec<_>>()
    }
}
