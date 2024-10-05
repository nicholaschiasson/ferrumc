pub mod errors;
mod loading;

use crate::errors::PluginsError;
use jni::objects::JObject;
use jni::{InitArgsBuilder, JNIVersion, JavaVM};
use std::sync::Arc;
use tracing::{error, info};
use which::which;

pub fn load_plugins() -> Result<Arc<JavaVM>, PluginsError> {
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
        .option("-Xcheck:jni")
        .build()
        .map_err(|e| PluginsError::JVMError(format!("Failed to create JVM args: {}", e)))?;
    let jvm = Arc::new(
        JavaVM::new(jvm_args)
            .map_err(|e| PluginsError::JVMError(format!("Failed to create JVM: {}", e)))?,
    );
    jvm.attach_current_thread_permanently()
        .map_err(|e| PluginsError::JVMError(format!("Failed to attach to JVM: {}", e)))?;
    let class_files = loading::get_class_files()?;
    let mut env = jvm.get_env().unwrap();
    for (class_data, class_name) in class_files {
        env.define_unnamed_class(&JObject::null(), &class_data)
            .map_err(|e| PluginsError::JVMError(format!("Failed to load class: {}", e)))?;
        info!("Loaded: {}", class_name);
    }
    Ok(jvm.clone())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_setup() {
        let jvm = super::load_plugins().unwrap();
        let mut env = jvm.get_env().unwrap();
        env.call_static_method("com/ferrumc/MainKt", "setup", "()V", &[])
            .unwrap();
    }
}
