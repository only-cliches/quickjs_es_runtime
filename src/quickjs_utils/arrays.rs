use crate::eserror::EsError;
use crate::quickjsruntime::{OwnedValueRef, QuickJsRuntime};
use libquickjs_sys as q;

pub fn is_array(q_js_rt: &QuickJsRuntime, obj_ref: &OwnedValueRef) -> bool {
    let r = obj_ref.borrow_value();
    let val = unsafe { q::JS_IsArray(q_js_rt.context, *r) };
    val > 0
}

pub fn get_length(q_js_rt: &QuickJsRuntime, arr_ref: &OwnedValueRef) -> Result<u32, EsError> {
    let len_ref = crate::quickjs_utils::objects::get_property(q_js_rt, arr_ref, "length")?;

    let len = crate::quickjs_utils::primitives::to_i32(&len_ref)?;

    Ok(len as u32)
}

pub fn create_array(q_js_rt: &QuickJsRuntime) -> Result<OwnedValueRef, EsError> {
    let arr = unsafe { q::JS_NewArray(q_js_rt.context) };
    let arr_ref = OwnedValueRef::new(arr);
    if arr_ref.is_exception() {
        return Err(EsError::new_str("Could not create array in runtime"));
    }
    Ok(arr_ref)
}

pub fn set_element(
    q_js_rt: &QuickJsRuntime,
    array_ref: &OwnedValueRef,
    index: u32,
    entry_value_ref: OwnedValueRef,
) -> Result<(), EsError> {
    let mut entry_value_ref = entry_value_ref;

    let ret = unsafe {
        q::JS_DefinePropertyValueUint32(
            q_js_rt.context,
            *array_ref.borrow_value(),
            index,
            entry_value_ref.consume_value(),
            q::JS_PROP_C_W_E as i32,
        )
    };
    if ret < 0 {
        return Err(EsError::new_str("Could not append element to array"));
    }
    Ok(())
}

pub fn get_element(
    q_js_rt: &QuickJsRuntime,
    array_ref: &OwnedValueRef,
    index: u32,
) -> Result<OwnedValueRef, EsError> {
    let value_raw =
        unsafe { q::JS_GetPropertyUint32(q_js_rt.context, *array_ref.borrow_value(), index) };
    let ret = OwnedValueRef::new(value_raw);
    if ret.is_exception() {
        return Err(EsError::new_str("Could not build array"));
    }
    Ok(ret)
}