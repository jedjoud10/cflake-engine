use rapier3d::prelude::*;

// Physics simulation
pub struct PhysicsSimulation {
    pub islands: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
    pub ccd_solver: CCDSolver,
    pub query_pipeline: QueryPipeline,
    pub physics_pipeline: PhysicsPipeline,
    pub integration_parameters: IntegrationParameters,
    pub gravity: veclib::Vector3<f32>,
}

impl PhysicsSimulation {
    // Create a new physics simulation from scratch
    pub fn new() -> Self {
        Self {
            islands: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            bodies: RigidBodySet::new(),
            colliders: ColliderSet::new(),
            joints: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            physics_pipeline: PhysicsPipeline::new(),
            integration_parameters: IntegrationParameters::default(),
            gravity: veclib::vec3(0.0, -9.81, 0.0),
        }
    }
    // Step once through the simulation
    pub fn step(&mut self) {
        // Convert gravity
        let gravity = Vector::new(self.gravity.x, self.gravity.y, self.gravity.z);
        let physics_hooks = ();
        let events = ();
        // Step
        self.physics_pipeline.step(
            &gravity,
            &self.integration_parameters,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.ccd_solver,
            &physics_hooks,
            &events,
        );
    }
}
