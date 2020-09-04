use crate::quickjs_utils::{functions, objects, primitives};
use crate::quickjsruntime::{OwnedValueRef, QuickJsRuntime};
use libquickjs_sys as q;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CLASSNAME_CLASSID_MAPPINGS: RefCell<HashMap<String, i32>> = RefCell::new(HashMap::new());
}

#[cfg(test)]
pub mod tests {
    use crate::esruntime::EsRuntime;
    use crate::esscript::EsScript;
    use crate::quickjs_utils::functions::new_native_function;
    use crate::quickjs_utils::get_global;
    use crate::quickjs_utils::reflection::{
        constructor, finalizer, js_class_call, register_class_name,
    };
    use crate::quickjsruntime::make_cstring;
    use libquickjs_sys as q;
    use std::sync::Arc;
    use std::time::Duration;

    /*

    JSClassExoticMethods {
        pub get_own_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                desc: *mut JSPropertyDescriptor,
                obj: JSValue,
                prop: JSAtom,
            ) -> ::std::os::raw::c_int,
        >,
        pub get_own_property_names: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                ptab: *mut *mut JSPropertyEnum,
                plen: *mut u32,
                obj: JSValue,
            ) -> ::std::os::raw::c_int,
        >,
        pub delete_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                obj: JSValue,
                prop: JSAtom,
            ) -> ::std::os::raw::c_int,
        >,
        pub define_own_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                this_obj: JSValue,
                prop: JSAtom,
                val: JSValue,
                getter: JSValue,
                setter: JSValue,
                flags: ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int,
        >,
        pub has_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                obj: JSValue,
                atom: JSAtom,
            ) -> ::std::os::raw::c_int,
        >,
        pub get_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                obj: JSValue,
                atom: JSAtom,
                receiver: JSValue,
            ) -> JSValue,
        >,
        pub set_property: ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                obj: JSValue,
                atom: JSAtom,
                value: JSValue,
                receiver: JSValue,
                flags: ::std::os::raw::c_int,
            ) -> ::std::os::raw::c_int,
        >,

            pub type JSClassFinalizer =
            ::std::option::Option<unsafe extern "C" fn(rt: *mut JSRuntime, val: JSValue)>;
        pub type JSClassGCMark = ::std::option::Option<
            unsafe extern "C" fn(rt: *mut JSRuntime, val: JSValue, mark_func: JS_MarkFunc),
        >;
        pub type JSClassCall = ::std::option::Option<
            unsafe extern "C" fn(
                ctx: *mut JSContext,
                func_obj: JSValue,
                this_val: JSValue,
                argc: ::std::os::raw::c_int,
                argv: *mut JSValue,
                flags: ::std::os::raw::c_int,
            ) -> JSValue,
        >;
        #[repr(C)]
        #[derive(Debug, Copy, Clone)]
        pub struct JSClassDef {
            pub class_name: *const ::std::os::raw::c_char,
            pub finalizer: JSClassFinalizer,
            pub gc_mark: JSClassGCMark,
            pub call: JSClassCall,
            pub exotic: *mut JSClassExoticMethods,
        }

             */

    #[test]
    pub fn test_proxy() {
        let rt: Arc<EsRuntime> = crate::esruntime::tests::TEST_ESRT.clone();
        let io = rt.add_to_event_queue_sync(|q_js_rt| {
            //

            let mut c_id: u32 = 0;
            let class_id: u32 = unsafe { q::JS_NewClassID(&mut c_id) };
            log::trace!("got class id {}", class_id);
            register_class_name("TestClass", class_id as i32);

            let c_name = make_cstring("TestClass").ok().unwrap();

            let mut exotic = q::JSClassExoticMethods {
                get_own_property: None,
                get_own_property_names: None,
                delete_property: None,
                define_own_property: None,
                has_property: None,
                get_property: None,
                set_property: None,
            };

            let class_def = q::JSClassDef {
                class_name: c_name.as_ptr(),
                finalizer: Some(finalizer),
                gc_mark: None,
                call: Some(js_class_call),
                exotic: &mut exotic,
            };

            let res = unsafe { q::JS_NewClass(q_js_rt.runtime, class_id, &class_def) };
            log::trace!("new class res {}", res);

            // todo reg ClassName as native_func with constructor is true

            let constructor_ref =
                new_native_function(q_js_rt, "TestClass", Some(constructor), 1, true)
                    .ok()
                    .expect("shit failed yo");

            let global_ref = get_global(q_js_rt);
            crate::quickjs_utils::objects::set_property(
                q_js_rt,
                &global_ref,
                "TestClass",
                constructor_ref,
            )
            .ok()
            .expect("could not set prop");

            log::trace!("set prop done");

            let eval_res = q_js_rt.eval(EsScript::new(
                "TestClass.es".to_string(),
                "let i = new TestClass(1, true, 'abc'); console.log('i._ES_INSTANCE_ID_ = '+i._ES_INSTANCE_ID_); i = null;".to_string(),
            ));
            if eval_res.is_err() {
                log::trace!("{}", eval_res.err().unwrap());
            }

            log::trace!("aftert eval");

            q_js_rt.gc();

            log::trace!("aftert gc");

            std::thread::sleep(Duration::from_secs(2));

            true
        });

        std::thread::sleep(Duration::from_secs(2));
        assert!(io)
    }
}

fn register_class_name(class_name: &str, class_id: i32) {
    CLASSNAME_CLASSID_MAPPINGS.with(|rc: &RefCell<HashMap<String, i32>>| {
        let mappings = &mut *rc.borrow_mut();
        mappings.insert(class_name.to_string(), class_id);
    });
}

fn resolve_class_id(class_name: &str) -> i32 {
    CLASSNAME_CLASSID_MAPPINGS.with(|rc: &RefCell<HashMap<String, i32>>| {
        let mappings = &*rc.borrow();
        *mappings.get(class_name).unwrap()
    })
}

unsafe extern "C" fn constructor(
    ctx: *mut q::JSContext,
    this_val: q::JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut q::JSValue,
) -> q::JSValue {
    log::trace!("constructor called, this_tag={}", this_val.tag);

    // this is the function we created earlier (the constructor)
    // so classname = this.name;
    let this_ref = OwnedValueRef::new(this_val);
    QuickJsRuntime::do_with(|q_js_rt| {
        let name_ref = objects::get_property(q_js_rt, &this_ref, "name")
            .ok()
            .expect("name get failed");
        let class_name = functions::call_to_string(q_js_rt, &name_ref)
            .ok()
            .expect("name.toString failed");

        log::trace!("classname={}", class_name);
        let class_id = resolve_class_id(class_name.as_str());

        log::trace!("constructor called, class_id={}", class_id);
        let class_val: q::JSValue = q::JS_NewObjectClass(ctx, class_id as i32);

        let class_val_ref = OwnedValueRef::new_no_free(class_val);
        objects::set_property2(
            q_js_rt,
            &class_val_ref,
            "_ES_INSTANCE_ID_",
            primitives::from_i32(2581),
            0, // not configurable, writable or enumerable
        )
        .ok()
        .expect("could not set instance id");

        class_val
    })
}

unsafe extern "C" fn finalizer(_rt: *mut q::JSRuntime, _val: q::JSValue) {
    //todo
    log::trace!("finalizer called");
}

unsafe extern "C" fn js_class_call(
    _ctx: *mut q::JSContext,
    _func_obj: q::JSValue,
    _this_val: q::JSValue,
    _argc: ::std::os::raw::c_int,
    _argv: *mut q::JSValue,
    _flags: ::std::os::raw::c_int,
) -> q::JSValue {
    log::trace!("js_class_call called");
    //todo
    crate::quickjs_utils::new_null()
}