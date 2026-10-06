use crate::{
    DEBUG_LOG_GC,
    types::value::{
        class::{ObjBoundMethod, ObjClass, ObjInstance},
        closure::ObjClosure,
        function::ObjFunction,
        native::ObjNative,
        obj::HeapObj,
        upvalue::ObjUpvalue,
    },
};

use self::arena::Arena;

mod arena;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ObjId {
    kind: ObjKind,
    index: usize,
}

impl ObjId {
    pub fn null() -> Self {
        Self {
            kind: ObjKind::String,
            index: usize::MAX,
        }
    }

    pub fn is_null(&self) -> bool {
        self.index == usize::MAX
    }
}

impl std::fmt::Display for ObjId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_null() {
            write!(f, "ObjId(null)")
        } else {
            write!(f, "ObjId({:?}, {})", self.kind, self.index)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ObjKind {
    String,
    Function,
    Closure,
    Native,
    Upvalue,
    Class,
    Instance,
    BoundMethod,
}

#[derive(Default)]
pub struct Heap {
    strings: Arena<String>,
    functions: Arena<ObjFunction>,
    closures: Arena<ObjClosure>,
    natives: Arena<ObjNative>,
    upvalues: Arena<ObjUpvalue>,
    classes: Arena<ObjClass>,
    instances: Arena<ObjInstance>,
    bound_methods: Arena<ObjBoundMethod>,
}

impl Heap {
    pub fn allocate(&mut self, object: HeapObj) -> ObjId {
        let id = match object {
            HeapObj::String(value) => ObjId {
                kind: ObjKind::String,
                index: self.strings.allocate(value),
            },
            HeapObj::Function(value) => ObjId {
                kind: ObjKind::Function,
                index: self.functions.allocate(value),
            },
            HeapObj::Closure(value) => ObjId {
                kind: ObjKind::Closure,
                index: self.closures.allocate(value),
            },
            HeapObj::Native(value) => ObjId {
                kind: ObjKind::Native,
                index: self.natives.allocate(value),
            },
            HeapObj::Upvalue(value) => ObjId {
                kind: ObjKind::Upvalue,
                index: self.upvalues.allocate(value),
            },
            HeapObj::Class(value) => ObjId {
                kind: ObjKind::Class,
                index: self.classes.allocate(value),
            },
            HeapObj::Instance(value) => ObjId {
                kind: ObjKind::Instance,
                index: self.instances.allocate(value),
            },
            HeapObj::BoundMethod(value) => ObjId {
                kind: ObjKind::BoundMethod,
                index: self.bound_methods.allocate(value),
            },
        };

        if DEBUG_LOG_GC {
            println!(
                "{id} allocate {:?} for HeapObj",
                std::mem::size_of::<HeapObj>(),
            );
        }

        id
    }

    pub fn deallocate(&mut self, id: ObjId) {
        if self.contains(id) {
            match id.kind {
                ObjKind::String => self.strings.deallocate(id.index),
                ObjKind::Function => self.functions.deallocate(id.index),
                ObjKind::Closure => self.closures.deallocate(id.index),
                ObjKind::Native => self.natives.deallocate(id.index),
                ObjKind::Upvalue => self.upvalues.deallocate(id.index),
                ObjKind::Class => self.classes.deallocate(id.index),
                ObjKind::Instance => self.instances.deallocate(id.index),
                ObjKind::BoundMethod => self.bound_methods.deallocate(id.index),
            }
        }

        if DEBUG_LOG_GC {
            println!(
                "{id} deallocate {:?} for HeapObj",
                std::mem::size_of::<HeapObj>(),
            );
        }
    }

    pub fn contains(&self, id: ObjId) -> bool {
        if id.is_null() {
            return false;
        }

        match id.kind {
            ObjKind::String => self.strings.contains(id.index),
            ObjKind::Function => self.functions.contains(id.index),
            ObjKind::Closure => self.closures.contains(id.index),
            ObjKind::Native => self.natives.contains(id.index),
            ObjKind::Upvalue => self.upvalues.contains(id.index),
            ObjKind::Class => self.classes.contains(id.index),
            ObjKind::Instance => self.instances.contains(id.index),
            ObjKind::BoundMethod => self.bound_methods.contains(id.index),
        }
    }

    pub fn mark(&mut self, id: ObjId) -> bool {
        match id.kind {
            ObjKind::String => self.strings.mark(id.index),
            ObjKind::Function => self.functions.mark(id.index),
            ObjKind::Closure => self.closures.mark(id.index),
            ObjKind::Native => self.natives.mark(id.index),
            ObjKind::Upvalue => self.upvalues.mark(id.index),
            ObjKind::Class => self.classes.mark(id.index),
            ObjKind::Instance => self.instances.mark(id.index),
            ObjKind::BoundMethod => self.bound_methods.mark(id.index),
        }
    }

    pub fn is_marked(&self, id: ObjId) -> bool {
        match id.kind {
            ObjKind::String => self.strings.is_marked(id.index),
            ObjKind::Function => self.functions.is_marked(id.index),
            ObjKind::Closure => self.closures.is_marked(id.index),
            ObjKind::Native => self.natives.is_marked(id.index),
            ObjKind::Upvalue => self.upvalues.is_marked(id.index),
            ObjKind::Class => self.classes.is_marked(id.index),
            ObjKind::Instance => self.instances.is_marked(id.index),
            ObjKind::BoundMethod => self.bound_methods.is_marked(id.index),
        }
    }

    pub fn clear_mark(&mut self, id: ObjId) {
        match id.kind {
            ObjKind::String => self.strings.clear_mark(id.index),
            ObjKind::Function => self.functions.clear_mark(id.index),
            ObjKind::Closure => self.closures.clear_mark(id.index),
            ObjKind::Native => self.natives.clear_mark(id.index),
            ObjKind::Upvalue => self.upvalues.clear_mark(id.index),
            ObjKind::Class => self.classes.clear_mark(id.index),
            ObjKind::Instance => self.instances.clear_mark(id.index),
            ObjKind::BoundMethod => self.bound_methods.clear_mark(id.index),
        }
    }

    pub fn iter_ids(&self) -> Vec<ObjId> {
        let mut ids = Vec::new();
        for index in self.strings.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::String,
                index,
            });
        }
        for index in self.functions.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Function,
                index,
            });
        }
        for index in self.closures.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Closure,
                index,
            });
        }
        for index in self.natives.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Native,
                index,
            });
        }
        for index in self.upvalues.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Upvalue,
                index,
            });
        }
        for index in self.classes.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Class,
                index,
            });
        }
        for index in self.instances.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::Instance,
                index,
            });
        }
        for index in self.bound_methods.iter_indices() {
            ids.push(ObjId {
                kind: ObjKind::BoundMethod,
                index,
            });
        }
        ids
    }

    pub fn kind(&self, id: ObjId) -> ObjKind {
        id.kind
    }

    pub fn string(&self, id: ObjId) -> &String {
        &self.strings[id.index]
    }

    pub fn function(&self, id: ObjId) -> &ObjFunction {
        &self.functions[id.index]
    }

    pub fn function_mut(&mut self, id: ObjId) -> &mut ObjFunction {
        &mut self.functions[id.index]
    }

    pub fn closure(&self, id: ObjId) -> &ObjClosure {
        &self.closures[id.index]
    }

    pub fn closure_mut(&mut self, id: ObjId) -> &mut ObjClosure {
        &mut self.closures[id.index]
    }

    pub fn native(&self, id: ObjId) -> &ObjNative {
        &self.natives[id.index]
    }

    pub fn upvalue(&self, id: ObjId) -> &ObjUpvalue {
        &self.upvalues[id.index]
    }

    pub fn upvalue_mut(&mut self, id: ObjId) -> &mut ObjUpvalue {
        &mut self.upvalues[id.index]
    }

    pub fn class(&self, id: ObjId) -> &ObjClass {
        &self.classes[id.index]
    }

    pub fn class_mut(&mut self, id: ObjId) -> &mut ObjClass {
        &mut self.classes[id.index]
    }

    pub fn instance(&self, id: ObjId) -> &ObjInstance {
        &self.instances[id.index]
    }

    pub fn instance_mut(&mut self, id: ObjId) -> &mut ObjInstance {
        &mut self.instances[id.index]
    }

    pub fn bound_method(&self, id: ObjId) -> &ObjBoundMethod {
        &self.bound_methods[id.index]
    }

    pub fn bound_method_mut(&mut self, id: ObjId) -> &mut ObjBoundMethod {
        &mut self.bound_methods[id.index]
    }
}
