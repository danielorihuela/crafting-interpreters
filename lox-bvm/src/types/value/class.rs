use crate::{
    collections::hashtable::HashTable,
    types::value::{
        obj::{Obj, ObjType, allocate_object},
        string::ObjString,
    },
};

#[repr(C)]
pub struct ObjClass {
    obj: Obj,
    pub name: *mut ObjString,
}

impl ObjClass {
    pub fn new(objects: *mut *mut Obj, name: *mut ObjString) -> *mut ObjClass {
        allocate_class(objects, name)
    }
}

fn allocate_class(objects: *mut *mut Obj, name: *mut ObjString) -> *mut ObjClass {
    let class = allocate_object::<ObjClass>(ObjType::Class, objects);

    unsafe {
        (*class).name = name;
    }

    class
}

#[repr(C)]
pub struct ObjInstance {
    obj: Obj,
    pub class: *mut ObjClass,
    pub fields: HashTable,
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
