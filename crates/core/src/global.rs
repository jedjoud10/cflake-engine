// Global file that contains all world related functions
/* #region Entities */
// Add an entity to the world. Let's hope that this doesn't exceed the maximum theoretical number of entities, which is 18,446,744,073,709,551,615
pub fn entity_add() {}
// Remove an entity from the world, returning a WorldCommandStatus of Failed if we failed to do so
pub fn entity_remove() {}
/* #endregion */
/* #region Components */
// Get a component
pub fn component() {}
// Get a component mutably
pub fn component_mut() {}
// Each time we link a component to an entity, we don't actually add it, we wait until we add that entity to actually add the component
pub fn link_component() {}
// This is an alternate function that links the component directly, but it is slower than the original one
pub fn link_component_direct() {}

