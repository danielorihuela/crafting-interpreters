use crate::{
    collections::hashtable::HashTable,
    types::value::{
        ObjPtrTarget, Value,
        closure::ObjClosure,
        obj::{Obj, ObjType, allocate_object},
        string::ObjString,
    },
};

#[repr(C)]
pub struct ObjClass {
    obj: Obj,
    pub name: *mut ObjString,
    pub methods: HashTable,
}

impl ObjPtrTarget for ObjClass {}

impl Display for ObjClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", unsafe { (*self.name).to_string() })
    }
}
use std::fmt::Display;

impl ObjClass {
    pub fn new(objects: *mut *mut Obj, name: *mut ObjString) -> *mut ObjClass {
        allocate_class(objects, name)
    }
}

fn allocate_class(objects: *mut *mut Obj, name: *mut ObjString) -> *mut ObjClass {
    let class = allocate_object::<ObjClass>(ObjType::Class, objects);

    unsafe {
        (*class).name = name;
        (*class).methods = HashTable::new();
    }

    class
}

#[repr(C)]
pub struct ObjInstance {
    obj: Obj,
    pub class: *mut ObjClass,
    pub fields: HashTable,
}

impl ObjPtrTarget for ObjInstance {}

impl Display for ObjInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} instance", unsafe { (*self.class).to_string() })
    }
}

impl ObjInstance {
    pub fn new(objects: *mut *mut Obj, class: *mut ObjClass) -> *mut ObjInstance {
        allocate_instance(objects, class)
    }
}

fn allocate_instance(objects: *mut *mut Obj, class: *mut ObjClass) -> *mut ObjInstance {
    let instance = allocate_object::<ObjInstance>(ObjType::Instance, objects);

    unsafe {
        (*instance).class = class;
        (*instance).fields = HashTable::new();
    }

    instance
}

#[repr(C)]
pub struct ObjBoundMethod {
    obj: Obj,
    pub receiver: Value,
    pub method: *mut ObjClosure,
}

impl ObjPtrTarget for ObjBoundMethod {}

impl Display for ObjBoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let function = unsafe { (*self.method).function };
        if unsafe { (*function).name.is_null() } {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", unsafe { (*(*function).name).to_string() })
        }
    }
}

impl ObjBoundMethod {
    pub fn new(
        objects: *mut *mut Obj,
        receiver: Value,
        method: *mut ObjClosure,
    ) -> *mut ObjBoundMethod {
        allocate_bound_method(objects, receiver, method)
    }
}

fn allocate_bound_method(
    objects: *mut *mut Obj,
    receiver: Value,
    method: *mut ObjClosure,
) -> *mut ObjBoundMethod {
    let bound_method = allocate_object::<ObjBoundMethod>(ObjType::BoundMethod, objects);

    unsafe {
        (*bound_method).receiver = receiver;
        (*bound_method).method = method;
    }

    bound_method
}
