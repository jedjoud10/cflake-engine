use rapier3d::prelude::*;
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
}

impl Physics {
    pub(crate) fn new() -> Self {
        let gravity = vector![0.0, -9.81, 0.0];
        let mut rigid_body_set = RigidBodySet::new();
        let mut collider_set = ColliderSet::new();    


        let mut integration_parameters = IntegrationParameters::default();        
        integration_parameters.set_inv_dt(400.0);
        
        let mut physics_pipeline = PhysicsPipeline::new();
        let mut island_manager = IslandManager::new();
        let mut broad_phase = BroadPhase::new();
        let mut narrow_phase = NarrowPhase::new();
        let mut impulse_joint_set = ImpulseJointSet::new();
        let mut multibody_joint_set = MultibodyJointSet::new();
        let mut ccd_solver = CCDSolver::new();
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
        }
    }
}