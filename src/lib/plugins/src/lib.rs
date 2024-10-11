mod api;
pub mod errors;
mod loading;

use crate::api::setup_plugins;
use crate::errors::PluginsError;
use ferrumc_utils::root;
use jni::objects::{
    AsJArrayRaw, JByteArray, JByteBuffer, JClass, JObject, JPrimitiveArray, JValue,
};
use jni::sys::{jbyte, jlong};
use jni::{InitArgsBuilder, JNIEnv, JNIVersion, JavaVM};
use once_cell::sync::OnceCell;
use std::hash::DefaultHasher;
use std::hash::Hasher;
use std::sync::Arc;
use tracing::{error, info};
use which::which;

#[derive(Debug)]
pub(crate) struct Plugin {
    name: String,
    display_name: String,
    methods: Vec<String>,
    group_id: String,
    entry: String,
}

#[derive(Debug)]
pub struct Plugins {
    jvm: Arc<JavaVM>,
    plugins: Vec<Plugin>,
}

#[no_mangle]
extern "C" fn input_handler<'a>(env: JNIEnv<'a>, _: JClass, data: JByteArray) -> JByteArray<'a> {
    if data.is_null() {
        let error_msg = "Received null byte array";
        error!("{}", error_msg);
        let ret = env.byte_array_from_slice(error_msg.as_bytes()).unwrap();
        return ret;
    }
    return match env.convert_byte_array(data) {
        Ok(bytes) => {
            info!(
                "Recived data from plugin: {:?}",
                String::from_utf8(bytes).unwrap()
            );
            env.byte_array_from_slice(&[0]).unwrap()
        }
        Err(e) => {
            let error_msg = format!("Failed to convert byte array: {}", e);
            env.byte_array_from_slice(error_msg.as_bytes()).unwrap()
        }
    };
}

pub fn load_plugins() -> Result<Plugins, PluginsError> {
    if which("java").is_err() {
        error!("Java not found in PATH");
        return Err(PluginsError::JVMError("Java not found in PATH".to_string()));
    }
    // IF IT BREAKS
    // 1. Make sure you have java installed
    // 2. Make sure a full java install is in your PATH (You need more than just the 3 exes in
    // C:\Program Files (x86)\Common Files\Oracle\Java\javapath, check C:\Program Files\Java\jdk-23\bin
    // and maybe add it to your PATH if needed)
    // 3. Try setting JAVA_HOME to the path of your java install
    // Grab java from here: https://www.oracle.com/au/java/technologies/downloads/#jdk21-windows and
    // add it to your PATH
    // TODO: Make it try install Java it it can't find it
    let jvm_args = InitArgsBuilder::new()
        .version(JNIVersion::V8)
        .option(format!(
            "-Djava.class.path={}",
            root!(".etc\\plugins\\Demoplugin-1.0.jar"),
        ))
        .option("-Xcheck:jni")
        // .option("-verbose:jni")
        .build()
        .map_err(|e| PluginsError::JVMError(format!("Failed to create JVM args: {}", e)))?;
    let jvm = Arc::new(
        JavaVM::new(jvm_args)
            .map_err(|e| PluginsError::JVMError(format!("Failed to create JVM: {}", e)))?,
    );
    jvm.attach_current_thread_permanently()
        .map_err(|e| PluginsError::JVMError(format!("Failed to attach to JVM: {}", e)))?;
    let mut plugins = Plugins {
        jvm: jvm.clone(),
        plugins: vec![],
    };
    setup_plugins(&mut plugins);
    Ok(plugins)
}

#[cfg(test)]
mod tests {
    use jni::strings::JNIString;
    use jni::NativeMethod;
    use std::ffi::c_void;

    #[test]
    fn test_setup() {
        ferrumc_logging::init_logging();
        let plugins = super::load_plugins().unwrap();
        let mut env = plugins.jvm.get_env().unwrap();
        let handler = NativeMethod {
            name: JNIString::from("nativeCall"),
            sig: JNIString::from("([B)[B"),
            fn_ptr: super::input_handler as *mut c_void,
        };
        env.register_native_methods("com/ferrumc/entry", &[handler])
            .unwrap();
        env.call_static_method("com/ferrumc/entry", "setup", "()V", &[])
            .unwrap();
    }
}
