
/*



impl<'a> Descriptor for WindowEvent<'a> {
    type DynFunc = dyn Fn(&mut World, &mut WindowEvent);
}

impl<'a, 'p> Caller<'p> for WindowEvent<'a> where 'a: 'p {
    type Params = (&'p mut World, &'p mut WindowEvent<'a>);

    fn call(ptrs: &Vec<Box<Self::DynFunc>>, params: Self::Params) {
        todo!()
    }
}

impl<'a, F: Fn(&mut World, &mut WindowEvent<'_>) + 'static>
    Event<WindowEvent<'a>, (&mut World, &mut WindowEvent<'_>)> for F
{
    fn boxed(self) -> Box<<WindowEvent<'a> as Descriptor>::DynFunc> {
        Box::new(self)
    }
}


impl Events {
    // Get the registry of a specific descriptor from within the global events
    // This is the only way we can interface with the values stored within the event manager
    pub fn registry<'b, 'd, M: Descriptor<'d>>(&'b mut self) -> Registry<'b, 'd, M> {
        Descriptor::registry(self)
    }
}

// Init event marker(FnOnce, called at the start of the engine)
pub struct Init(());

impl Descriptor<'static> for Init {
    type DynFunc = dyn FnOnce(&mut World, &EventLoop<()>);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry {
            sorted: &mut events.init.sorted,
        }
    }
}

impl<'p> Caller<'static, 'p> for Init {
    type Params = (&'p mut World, &'p EventLoop<()>);

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        let world = params.0;
        let el = params.1;

        let vec = std::mem::take(registry.sorted);

        for (boxed, _) in vec {
            boxed(world, el);
        }
    }
}

impl<F: FnOnce(&mut World) + 'static> Event<'static, Init, &mut World> for F {
    fn boxed(self) -> Box<<Init as Descriptor<'static>>::DynFunc> {
        Box::new(|world, _| self(world))
    }
}

impl<F: FnOnce(&mut World, &EventLoop<()>) + 'static>
    Event<'static, Init, (&mut World, &EventLoop<()>)> for F
{
    fn boxed(self) -> Box<<Init as Descriptor<'static>>::DynFunc> {
        Box::new(self)
    }
}

// Update event marker (called each frame)
pub struct Update(());

impl Descriptor<'static> for Update {
    type DynFunc = dyn Fn(&mut World);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry {
            sorted: &mut events.update.sorted,
        }
    }
}

impl<'p> Caller<'static, 'p> for Update {
    type Params = &'p mut World;

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        for (boxed, _) in registry.sorted.iter() {
            boxed(params);
        }
    }
}

impl<F: Fn(&mut World) + 'static> Event<'static, Update, &mut World> for F {
    fn boxed(self) -> Box<<Update as Descriptor<'static>>::DynFunc> {
        Box::new(move |world| self(world))
    }
}

// Window event marker (called by glutin handler) (this makes it extremely pain since the window event contains a lifetime)
impl<'d> Descriptor<'d> for WindowEvent<'d> {
    type DynFunc = dyn Fn(&mut World, &mut WindowEvent);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'd, Self> {
        Registry {
            sorted: &mut events.window.sorted,
        }
    }
}

impl<'d, 'p> Caller<'d, 'p> for WindowEvent<'d>
where
    'd: 'p,
{
    type Params = (&'p mut World, &'p mut WindowEvent<'d>);

    fn call(registry: Registry<'_, 'd, Self>, params: Self::Params) {
        let world = params.0;
        let ev = params.1;

        for (boxed, _) in registry.sorted.iter() {
            boxed(world, ev);
        }
    }
}

impl<'d, F: Fn(&mut World, &mut WindowEvent<'_>) + 'static>
    Event<'d, WindowEvent<'d>, (&mut World, &mut WindowEvent<'_>)> for F
{
    fn boxed(self) -> Box<<WindowEvent<'d> as Descriptor<'d>>::DynFunc> {
        Box::new(self)
    }
}

// Device event marker (called by glutin handler)
impl Descriptor<'static> for DeviceEvent {
    type DynFunc = dyn Fn(&mut World, &DeviceEvent);

    fn registry<'b>(events: &'b mut Events) -> Registry<'b, 'static, Self> {
        Registry {
            sorted: &mut events.device.sorted,
        }
    }
}

impl<'p> Caller<'static, 'p> for DeviceEvent {
    type Params = (&'p mut World, &'p DeviceEvent);

    fn call(registry: Registry<'_, 'static, Self>, params: Self::Params) {
        let world = params.0;
        let event = params.1;

        for (boxed, _) in registry.sorted.iter() {
            boxed(world, event);
        }
    }
}

impl<F: Fn(&mut World, &DeviceEvent) + 'static>
    Event<'static, DeviceEvent, (&mut World, &DeviceEvent)> for F
{
    fn boxed(self) -> Box<<DeviceEvent as Descriptor<'static>>::DynFunc> {
        Box::new(move |world, event| self(world, event))
    }
}
*/