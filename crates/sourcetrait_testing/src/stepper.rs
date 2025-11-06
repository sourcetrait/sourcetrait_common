use crate::*;

/// Sequentially runs through a series of test steps.
/// 
/// [O]: Initialization options type  
/// [S]: Internal state type that is passed through each step  
/// [T]: End product type that is returned after all steps are executed  
pub struct Stepper<O, S, T> {
    name: &'static str,
    init_func: InitFn<O, S, T>,
    step_funcs: Arc<IndexMap<&'static str, StepFn<S, T>>>,
}

impl<O, S, T> Stepper<O, S, T> {
    /// Creates a new [StepperBuilder] for constructing [Stepper]s
    pub fn builder(name: &'static str) -> StepperBuilder<O, S, T> {
        StepperBuilder::new(name)
    }
    
    /// Runs the stepper through all steps
    pub fn run(&self, testable: Testable, options: O) -> T {
        let namepath = testable.namepath().to_string();
        log::debug!("Stepper '{}' {namepath}: init", self.name);
        
        let init_fn = self.init_func;
        let StepState(mut state, mut subject) = init_fn(&testable, options);
        
        for (step_name, step_fn) in &*self.step_funcs {
            log::debug!("Stepper '{}' {namepath}: {step_name}", self.name);
            StepState(state, subject) = step_fn(&testable, state, subject);
        }
        
        subject
    }
    
    /// Runs the stepper through a specific step
    pub fn run_thru(&mut self, step: &'static str, testable: Testable, options: O) -> T {
        let namepath = testable.namepath().to_string();
        log::debug!("Stepper '{}' {namepath}: init", self.name);
        
        let init_fn = self.init_func;
        let StepState(mut state, mut subject) = init_fn(&testable, options);
        
        if step == INIT {
            return subject;
        }
        
        for (step_name, step_fn) in &*self.step_funcs {
            log::debug!("Stepper '{}' {namepath}: {step_name}", self.name);
            StepState(state, subject) = step_fn(&testable, state, subject);
            
            if *step_name == step {
                break;
            }
        }
        
        subject
    }
}

/// The initialization function type for a [Stepper].
pub type InitFn<O, S, T> = fn(&Testable<'_, '_,'_>, O) -> StepState<S, T>;
/// Represents a step function type for a [Stepper].
pub type StepFn<S, T> = fn(&Testable<'_, '_,'_>, S, T) -> StepState<S, T>;

/// Represents the internal state of a Stepper
/// 
/// <S>: Internal state type that is passed through each step  
/// <T>: Product type that is returned after all steps are executed
pub struct StepState<S, T>(pub S, pub T);

impl<S, T> StepState<S, T> {
    pub fn state(self) -> S {
        self.0
    }
    
    pub fn subject(self) -> T {
        self.1
    }
}

/// Reserved name for the initialization step in a [Stepper].
pub const INIT: &'static str = "init";

/// Builder for [Stepper]
/// 
/// [O]: Initialization options type  
/// [S]: Internal state type that is passed through each step  
/// [T]: End product type that is returned after all steps are executed
pub struct StepperBuilder<O, S, T> {
    name: &'static str,
    init_func: Option<InitFn<O, S, T>>,
    steps_builder: Option<IndexMap<&'static str, StepFn<S, T>>>,
    steps: Option<Arc<IndexMap<&'static str, StepFn<S, T>>>>,
}

/// Prepares initialization and step functions for [Stepper]s.
/// 
/// Designed to be used statically with [LazyLock].  
/// Call [StepperBuilder::finalize] to complete the setup.
impl<O, S, T> StepperBuilder<O, S, T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            init_func: None,
            steps_builder: Some(IndexMap::new()),
            steps: None,
        }
    }
    
    /// Sets the initialization function for the Stepper
    pub fn init(mut self, step: InitFn<O, S, T>) -> Self {
        self.init_func = Some(step);
        self
    }

    /// Adds a named step for the Stepper 
    pub fn step(mut self, name: &'static str, step: StepFn<S, T>) -> Self {
        debug_assert!(name != INIT, "Cannot use 'init' as a step name");
        self.steps_builder.as_mut().expect("builder phase").insert(name, step);
        self
    }
    
    /// Finishes setup of the builder
    pub fn finalize(mut self) -> Self {
        if self.init_func.is_none() {
            panic!("Stepper must have an initialization function defined");
        } else if self.steps_builder.is_none() {
            panic!("Stepper finalize already called");
        }
        
        self.steps = Some(Arc::new(self.steps_builder.take().expect("finalized")));
        self
    }
    
    /// Creates a new [Stepper]
    pub fn build(&self) -> Stepper<O, S, T> {
        let init_func = self.init_func
            .expect("Stepper must have an initialization function defined");
        let step_funcs = Arc::clone(self.steps.as_ref().expect("Stepper must be finalized"));
        
        Stepper {
            name: self.name,
            init_func,
            step_funcs,
        }
    }
    
    /// Creates a new [Stepper] and runs it with the provided testable and options
    pub fn run(&self, testable: Testable, options: O) -> T {
        self.build()
            .run(testable, options)
    }
    
    /// Creates a new [Stepper] and runs it through the specified step
    pub fn run_thru(&mut self, step: &'static str, testable: Testable, options: O) -> T {
        self.build()
            .run_thru(step, testable, options)
    }
}    
