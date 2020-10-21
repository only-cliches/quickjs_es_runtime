// store in thread_local

use crate::eserror::EsError;
use crate::esruntime::EsRuntime;
use crate::esscript::EsScript;
use crate::quickjs_utils::{errors, functions, gc, modules, objects, promises};
use crate::valueref::{JSValueRef, TAG_EXCEPTION};
use hirofa_utils::auto_id_map::AutoIdMap;
use libquickjs_sys as q;
use std::cell::RefCell;
use std::ffi::CString;
use std::panic;
use std::panic::catch_unwind;
use std::sync::{Arc, Weak};

pub type ModuleScriptLoader = dyn Fn(&str, &str) -> Option<EsScript> + Send + Sync + 'static;

thread_local! {
   /// the thread-local QuickJsRuntime
   /// this only exists for the worker thread of the EsEventQueue
   pub(crate) static QJS_RT: RefCell<QuickJsRuntime> = RefCell::new(QuickJsRuntime::new());

}

pub struct QuickJsRuntime {
    pub(crate) runtime: *mut q::JSRuntime,
    pub(crate) context: *mut q::JSContext,
    pub(crate) module_script_loader: Option<Box<ModuleScriptLoader>>,

    object_cache: RefCell<AutoIdMap<JSValueRef>>,
    es_rt_ref: Option<Weak<EsRuntime>>,
}

impl QuickJsRuntime {
    pub(crate) fn init_rt_ref(&mut self, rt_ref: Arc<EsRuntime>) {
        self.es_rt_ref = Some(Arc::downgrade(&rt_ref));
    }
    pub fn get_rt_ref(&self) -> Option<Arc<EsRuntime>> {
        if let Some(rt_ref) = &self.es_rt_ref {
            rt_ref.upgrade()
        } else {
            None
        }
    }
    fn new() -> Self {
        log::trace!("creating new QuickJsRuntime");
        let runtime = unsafe { q::JS_NewRuntime() };
        if runtime.is_null() {
            panic!("RuntimeCreationFailed");
        }

        // Configure memory limit if specified.
        //let memory_limit = None;
        //if let Some(limit) = memory_limit {
        //  unsafe {
        //q::JS_SetMemoryLimit(runtime, limit as _);
        //}
        //}

        let context = unsafe { q::JS_NewContext(runtime) };
        if context.is_null() {
            unsafe {
                q::JS_FreeRuntime(runtime);
            }
            panic!("ContextCreationFailed");
        }

        let q_rt = Self {
            runtime,
            context,
            module_script_loader: None,
            object_cache: RefCell::new(AutoIdMap::new_with_max_size(i32::MAX as usize)),
            es_rt_ref: None,
        };

        modules::set_module_loader(&q_rt);
        promises::init_promise_rejection_tracker(&q_rt);

        q_rt
    }

    pub fn call_function(
        &self,
        namespace: Vec<&str>,
        func_name: &str,
        arguments: Vec<JSValueRef>,
    ) -> Result<JSValueRef, EsError> {
        let namespace_ref = objects::get_namespace(self, namespace, false)?;
        functions::invoke_member_function(self, &namespace_ref, func_name, arguments)
    }

    pub fn gc(&self) {
        gc(self);
    }

    pub fn eval(&self, script: EsScript) -> Result<JSValueRef, EsError> {
        let filename_c = make_cstring(script.get_path())?;
        let code_c = make_cstring(script.get_code())?;

        log::debug!("q_js_rt.eval file {}", script.get_path());

        let value_raw = unsafe {
            q::JS_Eval(
                self.context,
                code_c.as_ptr(),
                script.get_code().len() as _,
                filename_c.as_ptr(),
                q::JS_EVAL_TYPE_GLOBAL as i32,
            )
        };

        log::trace!("after eval, checking error");

        // check for error
        let ret = JSValueRef::new(
            value_raw,
            false,
            true,
            format!("eval result of {}", script.get_path()).as_str(),
        );
        if ret.is_exception() {
            let ex_opt = self.get_exception();
            if let Some(ex) = ex_opt {
                Err(ex)
            } else {
                Err(EsError::new_str("eval failed and could not get exception"))
            }
        } else {
            while self.has_pending_jobs() {
                self.run_pending_job()?;
            }

            Ok(ret)
        }
    }

    pub fn eval_module(&self, script: EsScript) -> Result<JSValueRef, EsError> {
        log::debug!("q_js_rt.eval_module file {}", script.get_path());

        let filename_c = make_cstring(script.get_path())?;
        let code_c = make_cstring(script.get_code())?;

        let value_raw = unsafe {
            q::JS_Eval(
                self.context,
                code_c.as_ptr(),
                script.get_code().len() as _,
                filename_c.as_ptr(),
                q::JS_EVAL_TYPE_MODULE as i32,
            )
        };

        // check for error
        let ret = JSValueRef::new(
            value_raw,
            false,
            true,
            format!("eval_module result of {}", script.get_path()).as_str(),
        );

        log::trace!("evalled module yielded a {}", ret.borrow_value().tag);

        if ret.is_exception() {
            let ex_opt = self.get_exception();
            if let Some(ex) = ex_opt {
                Err(ex)
            } else {
                Err(EsError::new_str(
                    "eval_module failed and could not get exception",
                ))
            }
        } else {
            while self.has_pending_jobs() {
                self.run_pending_job()?;
            }
            Ok(ret)
        }
    }

    pub fn do_with<C, R>(task: C) -> R
    where
        C: FnOnce(&QuickJsRuntime) -> R,
    {
        QJS_RT.with(|qjs_rc| {
            let qjs_rt = &*qjs_rc.borrow();
            task(qjs_rt)
        })
    }

    /// throw an internal error to quickjs and create a new ex obj
    pub fn report_ex(&self, err: &str) -> q::JSValue {
        let c_err = CString::new(err);
        unsafe { q::JS_ThrowInternalError(self.context, c_err.as_ref().ok().unwrap().as_ptr()) };
        q::JSValue {
            u: q::JSValueUnion { int32: 0 },
            tag: TAG_EXCEPTION,
        }
    }

    /// Get the last exception from the runtime, and if present, convert it to a EsError.
    pub fn get_exception(&self) -> Option<EsError> {
        errors::get_exception(self)
    }

    pub fn has_pending_jobs(&self) -> bool {
        let flag = unsafe { q::JS_IsJobPending(self.runtime) };
        flag > 0
    }

    pub fn run_pending_job(&self) -> Result<(), EsError> {
        let flag = unsafe {
            let wrapper_mut = self as *const Self as *mut Self;
            let ctx_mut = &mut (*wrapper_mut).context;
            q::JS_ExecutePendingJob(self.runtime, ctx_mut)
        };
        if flag < 0 {
            let e = self
                .get_exception()
                .unwrap_or_else(|| EsError::new_str("Unknown exception while running pending job"));
            return Err(e);
        }
        Ok(())
    }

    pub fn cache_object(&self, obj: JSValueRef) -> i32 {
        let cache_map = &mut *self.object_cache.borrow_mut();
        cache_map.insert(obj) as i32
    }

    pub fn consume_cached_obj(&self, id: i32) -> JSValueRef {
        let cache_map = &mut *self.object_cache.borrow_mut();
        cache_map.remove(&(id as usize))
    }

    pub fn with_cached_obj<C, R>(&self, id: i32, consumer: C) -> R
    where
        C: FnOnce(&JSValueRef) -> R,
    {
        let cache_map = &*self.object_cache.borrow();
        let opt = cache_map.get(&(id as usize));
        consumer(opt.expect("no such obj in cache"))
    }
}

impl Drop for QuickJsRuntime {
    fn drop(&mut self) {
        log::trace!("before JS_FreeContext");
        unsafe { q::JS_FreeContext(self.context) };

        log::trace!("before JS_FreeRuntime");
        unsafe { q::JS_FreeRuntime(self.runtime) };

        log::error!("error while free runtime");

        log::trace!("after drop QuickJsRuntime");
    }
}

/// Helper for creating CStrings.
pub(crate) fn make_cstring(value: &str) -> Result<CString, EsError> {
    let res = CString::new(value);
    match res {
        Ok(val) => Ok(val),
        Err(_) => Err(EsError::new_string(format!(
            "could not create cstring from {}",
            value
        ))),
    }
}

#[cfg(test)]
pub mod tests {
    use crate::esscript::EsScript;
    use crate::quickjsruntime::QuickJsRuntime;

    #[test]
    fn test_rt() {
        log::info!("> test_rt");

        let rt = QuickJsRuntime::new();
        rt.eval(EsScript::new("test.es", "1+1;"))
            .ok()
            .expect("could not eval");

        log::info!("< test_rt");
    }
}
