pub mod errors;
use jni::{JavaVM, InitArgsBuilder, JNIVersion};
use jni::objects::JObject;
use ferrumc_utils::root;

pub fn load_plugins() {
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
    .unwrap();
    let jvm = JavaVM::new(jvm_args).unwrap();
    let mut env = jvm.attach_current_thread().unwrap();
    // Get all the class file in root!(".etc/plugins") and load them
    let class_files = std::fs::read_dir(root!(".etc/plugins")).unwrap();
    for dir_file in class_files.flatten() {
            let file_name = dir_file.file_name();
            let file_name = file_name.to_str().unwrap();
            if file_name.ends_with(".class") {
                let data = std::fs::read(dir_file.path()).unwrap();
                env.define_class(file_name, &JObject::null(), &data).unwrap();
                println!("Loaded: {}", file_name);
            }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_setup() {
        super::load_plugins();
    }
}
