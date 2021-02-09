//! Map utils, these methods can be used to manage Map objects from rust
//! see [MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map) for more on Maps

use crate::eserror::EsError;
use crate::quickjs_utils::objects::construct_object;
use crate::quickjs_utils::{functions, get_constructor, objects, primitives};
use crate::quickjscontext::QuickJsContext;
use crate::valueref::JSValueRef;
use libquickjs_sys as q;

/// create new instance of Map
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::new_map_q;
/// use quickjs_runtime::valueref::JSValueRef;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
/// });
/// ```
pub fn new_map_q(q_ctx: &QuickJsContext) -> Result<JSValueRef, EsError> {
    unsafe { new_map(q_ctx.context) }
}

/// create new instance of Map
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn new_map(ctx: *mut q::JSContext) -> Result<JSValueRef, EsError> {
    let map_constructor = get_constructor(ctx, "Map")?;
    construct_object(ctx, &map_constructor, vec![])
}

/// set a key/value pair in a Map
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::{new_map_q, set_q};
/// use quickjs_runtime::valueref::JSValueRef;
/// use quickjs_runtime::quickjs_utils::primitives;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
///    let key = primitives::from_i32(12);
///    let value = primitives::from_i32(23);
///    set_q(q_ctx, &my_map, key, value).ok().unwrap();
/// });
/// ```
pub fn set_q(
    q_ctx: &QuickJsContext,
    map: &JSValueRef,
    key: JSValueRef,
    val: JSValueRef,
) -> Result<JSValueRef, EsError> {
    unsafe { set(q_ctx.context, map, key, val) }
}

/// set a key/value pair in a Map
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn set(
    ctx: *mut q::JSContext,
    map: &JSValueRef,
    key: JSValueRef,
    val: JSValueRef,
) -> Result<JSValueRef, EsError> {
    functions::invoke_member_function(ctx, map, "set", vec![key, val])
}

/// get a value from a map by key
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::{new_map_q, get_q, set_q};
/// use quickjs_runtime::valueref::JSValueRef;
/// use quickjs_runtime::quickjs_utils::primitives;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
///    let key = primitives::from_i32(12);
///    let value = primitives::from_i32(23);
///    set_q(q_ctx, &my_map, key.clone(), value).ok().unwrap();
///    let val_res = get_q(q_ctx, &my_map, key).ok().unwrap();
///    assert_eq!(primitives::to_i32(&val_res).ok().unwrap(), 23);
/// });
/// ```
pub fn get_q(
    q_ctx: &QuickJsContext,
    map: &JSValueRef,
    key: JSValueRef,
) -> Result<JSValueRef, EsError> {
    unsafe { get(q_ctx.context, map, key) }
}

/// get a value from a map by key
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn get(
    ctx: *mut q::JSContext,
    map: &JSValueRef,
    key: JSValueRef,
) -> Result<JSValueRef, EsError> {
    functions::invoke_member_function(ctx, map, "get", vec![key])
}

/// delete a value from a map by key
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::{new_map_q, set_q, delete_q};
/// use quickjs_runtime::valueref::JSValueRef;
/// use quickjs_runtime::quickjs_utils::primitives;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
///    let key = primitives::from_i32(12);
///    let value = primitives::from_i32(23);
///    set_q(q_ctx, &my_map, key.clone(), value).ok().unwrap();
///    delete_q(q_ctx, &my_map, key).ok().unwrap();
/// });
/// ```
pub fn delete_q(
    q_ctx: &QuickJsContext,
    map: &JSValueRef,
    key: JSValueRef,
) -> Result<bool, EsError> {
    unsafe { delete(q_ctx.context, map, key) }
}

/// delete a value from a map by key
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn delete(
    ctx: *mut q::JSContext,
    map: &JSValueRef,
    key: JSValueRef,
) -> Result<bool, EsError> {
    let res = functions::invoke_member_function(ctx, map, "delete", vec![key])?;
    primitives::to_bool(&res)
}

/// check whether a Map has a value for a key
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::{new_map_q, set_q, has_q};
/// use quickjs_runtime::valueref::JSValueRef;
/// use quickjs_runtime::quickjs_utils::primitives;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
///    let key = primitives::from_i32(12);
///    let value = primitives::from_i32(23);
///    set_q(q_ctx, &my_map, key.clone(), value).ok().unwrap();
///    let bln_has = has_q(q_ctx, &my_map, key).ok().unwrap();
///    assert!(bln_has);
/// });
/// ```
pub fn has_q(q_ctx: &QuickJsContext, map: &JSValueRef, key: JSValueRef) -> Result<bool, EsError> {
    unsafe { has(q_ctx.context, map, key) }
}

/// check whether a Map has a value for a key
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn has(
    ctx: *mut q::JSContext,
    map: &JSValueRef,
    key: JSValueRef,
) -> Result<bool, EsError> {
    let res = functions::invoke_member_function(ctx, map, "has", vec![key])?;
    primitives::to_bool(&res)
}

/// get the number of entries in a map
/// # Example
/// ```rust
/// use quickjs_runtime::esruntimebuilder::EsRuntimeBuilder;
/// use quickjs_runtime::quickjs_utils::maps::{new_map_q, set_q, size_q};
/// use quickjs_runtime::valueref::JSValueRef;
/// use quickjs_runtime::quickjs_utils::primitives;
///
/// let rt = EsRuntimeBuilder::new().build();
/// rt.add_to_event_queue_sync(|q_js_rt| {
///    let q_ctx = q_js_rt.get_main_context();
///    let my_map: JSValueRef = new_map_q(q_ctx).ok().unwrap();
///    let key = primitives::from_i32(12);
///    let value = primitives::from_i32(23);
///    set_q(q_ctx, &my_map, key.clone(), value).ok().unwrap();
///    let i_size = size_q(q_ctx, &my_map).ok().unwrap();
///    assert_eq!(i_size, 1);
/// });
/// ```
pub fn size_q(q_ctx: &QuickJsContext, map: &JSValueRef) -> Result<i32, EsError> {
    unsafe { size(q_ctx.context, map) }
}

/// get the number of entries in a map
/// # Safety
/// please ensure the passed JSContext is still valid
pub unsafe fn size(ctx: *mut q::JSContext, map: &JSValueRef) -> Result<i32, EsError> {
    let res = objects::get_property(ctx, &map, "size")?;
    primitives::to_i32(&res)
}

// todo, clear, forEach, keys, values, entries

#[cfg(test)]
pub mod tests {
    use crate::esruntime::EsRuntime;
    use crate::quickjs_utils::maps::{new_map_q, set_q};
    use crate::quickjs_utils::primitives;
    use std::sync::Arc;

    #[test]
    fn test_map() {
        let rt: Arc<EsRuntime> = crate::esruntime::tests::TEST_ESRT.clone();
        rt.add_to_event_queue_sync(|q_js_rt| {
            let q_ctx = q_js_rt.get_main_context();
            let map = new_map_q(q_ctx).ok().expect("map creation failed");
            let key = primitives::from_i32(12);
            let val = primitives::from_i32(23);
            set_q(q_ctx, &map, key, val).ok().expect("set failed");
        });
    }
}
