use crate::*;

pub trait UiComponentTrait: Sized {
    /// Is this process being ran from the command-line / a terminal?
    fn has_command_line(&self) -> CrossResult<bool>;
    
    /// Does this process have access to a window manager session?
    fn has_graphical(&self) -> CrossResult<bool>;
    
    /// Sets the application's preference for either CLI or GUI when
    /// executing commands.
    fn prefer(&self, ui: UserInterface) -> CrossResult<()>;
    
    /// Retrieves the application's preference for CLI or GUI commands.
    fn preference(&self) -> CrossResult<UserInterface>;

    /// Is the application's preference the same as what is specified?
    fn prefers(&self, ui: UserInterface) -> CrossResult<bool>;
}

#[allow(private_bounds)]
pub struct StandardUiComponent<L: UiComponentLookup>(pub(crate) L);

#[allow(private_bounds)]
impl<L: UiComponentLookup> StandardUiComponent<L> {
    fn lookup(&self) -> &L { &self.0 }
}

impl<L: UiComponentLookup> UiComponentTrait for StandardUiComponent<L> {
    fn has_command_line(&self) -> CrossResult<bool> {
        let has_command_line = {
            let mut ui_cache_lock = ui_cache_lock()?;
            cache_locked_value_mut(&mut ui_cache_lock)?
                .has_command_line
                .determine(|| {
                    self.lookup().lookup_has_command_line()
                        .map_err(CrossError::from)
                })
                .map(|v| *v)
        };
        
        has_command_line
    }

    fn has_graphical(&self) -> CrossResult<bool> {
        let has_graphical = {
            let mut ui_cache_lock = ui_cache_lock()?;
            cache_locked_value_mut(&mut ui_cache_lock)?
                .has_graphical
                .determine(|| {
                    self.lookup().lookup_has_graphical()
                        .map_err(CrossError::from)
                })
                .map(|v| *v)
        };
        
        has_graphical
    }
    
    fn prefer(&self, ui: UserInterface) -> CrossResult<()> {
        if ui == UserInterface::GUI && !self.has_graphical()? {
            // - cli can possibly be started by launching a terminal app
            // - gui would require launching an *entire* window manager
            Err(CrossError::Unavailable)
        } else {
            let result = {
                let mut ui_cache_lock = ui_cache_lock()?;
                cache_locked_value_mut(&mut ui_cache_lock)?
                    .preference.set(ui)
            };
            
            result
        }
    }
    
    fn preference(&self) -> CrossResult<UserInterface> {
        let preference = {
            let this = self;
            let mut ui_cache_lock = ui_cache_lock()?;
            cache_locked_value_mut(&mut ui_cache_lock)?
                .preference
                .determine(|| Ok(
                    match ( 
                        this.lookup().lookup_has_command_line().unwrap_or(false),
                        this.lookup().lookup_has_graphical().unwrap_or(false)
                    ) {
                        (true, true) => UserInterface::CLI,
                        (false, true) => UserInterface::GUI,
                        (_, false) => UserInterface::CLI,
                    }
                ))
                .map(|v| *v)
        };
        
        preference
    }

    fn prefers(&self, ui: UserInterface) -> CrossResult<bool> {
        Ok(self.preference()? == ui)
    }
    
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum UserInterface {
    #[default]
    None,
    CLI,
    GUI,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct UiCache {
    has_command_line: CacheDetermined<bool>,
    has_graphical: CacheDetermined<bool>,
    preference: CacheDetermined<UserInterface>,
}

impl UiCache {
    pub(crate) const fn default_const() -> Self {
        Self {
           has_command_line: None,
           has_graphical: None, 
           preference: None,
        }
    }
}

fn ui_cache() -> &'static StaticCache<UiCache> {
    static CACHE: LazyLock<StaticCache<UiCache>> = LazyLock::new(|| { 
        new_static_cache_value(UiCache::default_const())
    });
    
    &CACHE
}

pub(crate) fn ui_cache_lock<'lock>() -> CrossResult<StaticCacheLock<'lock, UiCache>> {
    ui_cache().lock().map_err(|_| CrossError::lock(CrossErr::UiCache))
}
