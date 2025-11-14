use crate::noita::types::*;
// This only stores pointers that are constant, so should be safe to share between threads.
unsafe impl Sync for Globals {}
unsafe impl Send for Globals {}
macro_rules! maybe_ptr {
    (true, $ty:ty) => {*const *mut $ty};
    (false, $ty:ty) => {*mut $ty};
}
macro_rules! maybe_ptr_get {
    (true, $name:tt, $self:ident) => {
        unsafe { $self.$name.as_ref().unwrap().as_ref().unwrap() }
    };
    (false, $name:tt, $self:ident) => {
        unsafe { $self.$name.as_ref().unwrap() }
    };
}
macro_rules! maybe_ptr_get_mut {
    (true, $name:tt, $self:ident) => {
        unsafe { $self.$name.as_ref().unwrap().as_mut().unwrap() }
    };
    (false, $name:tt, $self:ident) => {
        unsafe { $self.$name.as_mut().unwrap() }
    };
}
macro_rules! make_globals {
    ($(($name:tt, $addr:expr, $ptr:tt, $ty:ty)),*) => {
        #[derive(Debug)]
        pub struct Globals {
            $(
                pub $name: maybe_ptr!($ptr, $ty),
            )*
        }
        impl Default for Globals {
            fn default() -> Self {
                Self {
                    $(
                        $name: $addr as maybe_ptr!($ptr, $ty),
                    )*
                }
            }
        }
        #[derive(Debug)]
        pub struct GlobalsRef {
            $(
                pub $name: &'static $ty,
            )*
        }
        #[derive(Debug)]
        pub struct GlobalsMut {
            $(
                pub $name: &'static mut $ty,
            )*
        }
        impl Globals {
            pub fn as_ref(&self) -> GlobalsRef {
                GlobalsRef {
                    $(
                        $name: self.$name(),
                    )*
                }
            }
            pub fn as_mut(&mut self) -> GlobalsMut {
                paste!{
                    GlobalsMut {
                        $(
                            $name: self.[<$name _mut>](),
                        )*
                    }
                }
            }
            $(
                pub fn $name(&self) -> &'static $ty {
                    maybe_ptr_get!($ptr, $name, self)
                }
            )*
            $(
                paste! {
                    pub fn [<$name _mut>](&mut self) -> &'static mut $ty {
                        maybe_ptr_get_mut!($ptr, $name, self)
                    }
                }
            )*
        }
    };
}
use paste::paste;
make_globals!(
    (entity_manager, 0x01204b98, true, EntityManager),
    (world_seed, 0x01205004, false, usize),
    (new_game_count, 0x01205024, false, usize),
    (global_stats, 0x01208940, false, GlobalStats),
    (game_global, 0x0122374c, true, GameGlobal),
    (entity_tag_manager, 0x01206fac, true, TagManager<u16>),
    (component_type_manager, 0x01223c88, false, ComponentTypeManager),
    (component_tag_manager, 0x01204b30, true, TagManager<u8>),
    (translation_manager, 0x01207c28, false, TranslationManager),
    (platform, 0x01221bc0, false, Platform),
    (filenames, 0x01207bd4, false, StdVec<StdString>),
    (inventory, 0x012224f0, false, Inventory),
    (mods, 0x01207e90, false, Mods),
    (max_component, 0x01152ff0, false, usize),
    (component_manager, 0x012236e8, false, ComponentSystemManager),
    (world_state, 0x01204bd0, true, Entity),
    (world_state_component, 0x01205010, true, WorldStateComponent),
    (event_manager, 0x01204b34, true, EventManager),
    (death_match, 0x01204bc0, true, DeathMatch)
);
