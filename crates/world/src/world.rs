use std::{any::Any, ptr::NonNull, ffi::c_void};

use ahash::AHashMap;
use gui::egui::util::id_type_map::TypeId;

// A resource is some arbitrary unique data that will be shared between multiple systems
pub trait Resource: Any {}

// A "world" simply contains multiple resources
pub struct World(AHashMap<TypeId, Box<dyn Any>>);

impl World {
    // Insert a new resource into the world
    // If the resource already exists, it will overwrite it
    fn insert<R: Resource>(&mut self, resource: R) {
        let id = TypeId::of::<R>();
        let boxed = Box::new(resource);
        self.0.insert(id, boxed);
    }
    
    // Remove a resource from the world
    fn remove<R: Resource>(&mut self) -> Option<R> {
        let id = TypeId::of::<R>();
        let boxed = self.0.remove(&id)?;
        boxed.downcast::<R>().ok().map(|b| *b)
    }

    // Get an immutable resource reference
    fn get<R: Resource>(&self) -> Option<&R> {
        let id = TypeId::of::<R>();
        self.0.get(&id).map(|boxed| boxed.downcast_ref().unwrap())
    }
    
    // Get a mutable resource reference
    fn get_mut<R: Resource>(&mut self) -> Option<&mut R> {
        let id = TypeId::of::<R>();
        self.0.get_mut(&id).map(|boxed| boxed.downcast_mut().unwrap())
    }

    // Get the raw pointer to a resource, if possible
    // TODO: Find a way to fetch different types of resources without unsafe
    fn get_mut_ptr<R: Resource>(&mut self) -> Option<NonNull<R>> {
        self.get_mut().map(|x| NonNull::new(x as *mut R)).flatten()
    }
}

// A resource handle is just an abstraction over mutable and immutable references to resources
pub trait Handle<'a>: where Self: 'a {
    type Res: Resource;

    // Convert a raw pointer into the handle
    unsafe fn convert(ptr: NonNull<Self::Res>) -> Self;

    // Get the type id of the underlying resource
    fn id() -> TypeId {
        TypeId::of::<Self::Res>()
    }
}
impl<'a, R: Resource> Handle<'a> for &'a R {
    type Res = R;

    unsafe fn convert(ptr: NonNull<Self::Res>) -> Self {
        ptr.as_ref()
    }
}
impl<'a, R: Resource> Handle<'a> for &'a mut R {
    type Res = R;

    unsafe fn convert(mut ptr: NonNull<Self::Res>) -> Self {
        ptr.as_mut()
    }
}

// A system is a type of event that can execute each frame or during initialization.
// Systems can have multiple inputs, and those inputs are all stored within the world 
pub trait System<'a> {
    // Input resources handles tuple
    type Inputs: 'a;

    // Resource pointer tuple
    type Ptrs;

    // Fetch the resource pointers from the world
    fn fetch(world: &mut World) -> Option<Self::Ptrs>;

    // Convert the pointers to the input handles
    unsafe fn handles(ptrs: Self::Ptrs) -> Self::Inputs;

    // Get the TypeIds of the input resources
    fn ids() -> &'static [TypeId];

    // Execute the system using the given input resources handles
    fn execute(&mut self, handles: Self::Inputs);
}

impl<'a, A: Handle<'a>> System<'a> for fn(A) {
    type Inputs = A;
    type Ptrs = NonNull<A::Res>;

    fn fetch(world: &mut World) -> Option<Self::Ptrs> {
        world.get_mut_ptr()
    }

    unsafe fn handles(ptrs: Self::Ptrs) -> Self::Inputs {
        A::convert(ptrs)
    }

    fn execute(&mut self, handles: Self::Inputs) {
        self(handles)
    }

    fn ids() -> &'static [TypeId] {
        &[A::id()]
    }
}

impl<'a, A: Handle<'a>, B: Handle<'a>> System<'a> for fn(A, B) {
    type Inputs = (A, B);
    type Ptrs = (NonNull<A::Res>, NonNull<B::Res>);

    fn fetch(world: &mut World) -> Option<Self::Ptrs> {
        let a = world.get_mut_ptr::<A::Res>()?;
        let b = world.get_mut_ptr::<B::Res>()?;
        Some(a, b)
    }

    unsafe fn handles(ptrs: Self::Ptrs) -> Self::Inputs {
        A::convert(ptrs)
    }

    fn execute(&mut self, handles: Self::Inputs) {
        self(handles)
    }

    fn ids() -> &'static [TypeId] {
        &[A::id(), B::id()]
    }
}




/*
// The current state of the world
#[derive(Clone, Copy)]
pub enum WorldState {
    Init,
    Active,
    Exit,
}

// The whole world that stores our managers and data
#[derive(Getters, CopyGetters, Setters)]
pub struct World {
    // User
    pub input: InputManager,
    pub io: IOManager,

    // Rendering
    pub graphics: Graphics,
    pub ui: UserInterface,

    // Logic
    pub state: WorldState,
    pub ecs: EcsManager,
    pub events: SystemSet<Self>,
    pub resources: ResourcesSet,
    pub physics: PhysicsSimulation,

    // Other
    pub time: Time,
    pub settings: Settings,
    pub audio: AudioPlayer,
}

// World implementation
impl World {
    // Create a new world
    pub fn new(settings: Settings, io: IOManager, graphics: Graphics) -> Self {
        let gui = gui::UserInterface::new(&mut pipeline);
        let mut world = World {
            input: Default::default(),
            io,
            pipeline,
            renderer,
            gui,
            state: WorldState::Init,
            ecs: EcsManager::default(),
            events: Default::default(),
            resources: Default::default(),
            physics: PhysicsSimulation::new(),
            time: Default::default(),
            settings: Default::default(),
            audio: Default::default(),
        };
        // Just set the game settings and we are done
        world.settings = settings;
        println!("World init done!");
        world
    }
    // Called each frame
    pub fn update(&mut self, delta: f32) {
        self.state = WorldState::Running;
        // Update the timings
        self.time.update(delta);

        // Update game logic (this includes rendering the world)
        self.pipeline.start_frame(&mut self.renderer, self.time.delta(), self.time.elapsed());
        self.gui.begin_frame(self.pipeline.window().context().window());

        // Prepare the Ecs manager
        self.ecs.prepare();

        // Execute
        let systems = self.events.clone();
        systems.execute(self);

        // Late update
        self.pipeline.end_frame();
        self.input.late_update();
    }
    // We must destroy the world
    pub fn destroy(&mut self) {}
}
*/