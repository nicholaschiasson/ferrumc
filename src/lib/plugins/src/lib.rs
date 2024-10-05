pub mod errors;
use crate::errors::PluginsError;
use ferrumc_utils::root;
use jni::objects::JObject;
use jni::{AttachGuard, InitArgsBuilder, JNIEnv, JNIVersion, JavaVM};
use std::sync::Arc;
use tracing::error;
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
    let class_files = std::fs::read_dir(root!(".etc/plugins")).unwrap();
    let mut env = jvm.get_env().unwrap();
    for dir_file in class_files.flatten() {
        let file_name = dir_file.file_name();
        let file_name = file_name.to_str().unwrap();
        if file_name.ends_with(".class") {
            let data = std::fs::read(dir_file.path()).unwrap();
            env.define_unnamed_class(&JObject::null(), &data)
                .map_err(|e| PluginsError::JVMError(format!("Failed to load class: {}", e)))?;
            println!("Loaded: {}", file_name);
        }
    }
    Ok(jvm.clone())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_setup() {
        println!("Mem use before: {}", memory_stats::memory_stats().unwrap().physical_mem);
        let jvm = super::load_plugins().unwrap();
        println!("Mem use during: {}", memory_stats::memory_stats().unwrap().physical_mem);
        let mut env = jvm.get_env().unwrap();
        env.call_static_method("com/ferrumc/MainKt", "setup", "()V", &[])
            .unwrap();
        println!("Mem use after: {}", memory_stats::memory_stats().unwrap().physical_mem);
    }
}
