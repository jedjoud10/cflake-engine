use rapier3d::prelude::*;

// Main physics resource that contains all the Rapier3D data structures
// that are needed to simulate the physics engine
pub struct Physics {
    pub(crate) bodies: RigidBodySet,
    pub(crate) colliders: ColliderSet,
    pub(crate) integration_parameters: IntegrationParameters,
    pub(crate) physics_pipeline: PhysicsPipeline,
    pub(crate) islands: IslandManager,
    pub(crate) broad_phase: BroadPhase,
    pub(crate) narrow_phase: NarrowPhase,
    pub(crate) impulse_joints: ImpulseJointSet,
    pub(crate) multibody_joints: MultibodyJointSet,
    pub(crate) ccd_solver: CCDSolver,
    pub(crate) query: QueryPipeline,
    pub(crate) gravity: vek::Vec3<f32>,
}

impl Physics {
    pub(crate) fn new() -> Self {
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();

        let mut integration_parameters = IntegrationParameters::default();
        integration_parameters.set_inv_dt(utils::TICKS_PER_SEC as f32);
        //integration_parameters.allowed_linear_error = 0.0001;
        //integration_parameters.max_penetration_correction = 0.001;

        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = BroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
        let mut query = QueryPipeline::new();
        let physics_hooks = ();
        let event_handler = ();

        Self {
            bodies: rigid_body_set,
            colliders: collider_set,
            integration_parameters,
            physics_pipeline,
            islands: island_manager,
            broad_phase,
            narrow_phase,
            impulse_joints: impulse_joint_set,
            multibody_joints: multibody_joint_set,
            ccd_solver,
            query,
            gravity: vek::Vec3::new(0.0, -9.81, 0.0),
        }
    }

    pub(crate) fn step(&mut self) {
        let Physics {
            bodies,
            colliders,
            integration_parameters,
            physics_pipeline,
            islands,
            broad_phase,
            narrow_phase,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            query,
            gravity,
        } = self;
        let gravity = crate::util::vek_vec_to_na_vec(*gravity);

        physics_pipeline.step(
            &gravity,
            &integration_parameters,
            islands,
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            impulse_joints,
            multibody_joints,
            ccd_solver,
            Some(query),
            &(),
            &(),
        );
    }
}
