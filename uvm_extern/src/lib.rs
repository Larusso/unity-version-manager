extern crate flexi_logger;
extern crate jni;
#[macro_use]
extern crate log;
extern crate uvm_core;

mod install;

use jni::JNIEnv;
use jni::objects::{JClass, JString, JValue, JObject};
use jni::sys::{jstring,jboolean,jobject,jobjectArray};

use std::collections::HashSet;
use uvm_core::unity::Version;
use uvm_core::unity::Installation;
use uvm_core::install::InstallVariant;
use std::str::FromStr;
use std::path::Path;
use flexi_logger::{Logger, LogSpecification, Record, LevelFilter, Level};

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_UnityVersionManager_detectProjectVersion(env: JNIEnv, _class: JClass, path: JObject, recursive: jboolean) -> jstring {
    //setup_log();
    let path:JString = env.call_method(path, "getPath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap().into();

    let path:String = env.get_string(path).expect("Couldn't get java string!").into();
    let recursive:bool = recursive != 0;

    let p = Path::new(&path);

    let output:JObject = match uvm_core::dectect_project_version(p, Some(recursive)) {
        Ok(version) => env.new_string(version.to_string()).expect("Couldn't create java string!").into(),
        Err(_) => JObject::null()
    };

    output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_UnityVersionManager_detectEditorInstallation(env: JNIEnv, _class: JClass, version: JString) -> jobject {
    //setup_log();
    let version:String = env.get_string(version).expect("Couldn't get java string!").into();
    let class:JClass = env.find_class("java/io/File").expect("Can't find class.");

    let version = Version::from_str(&version).unwrap();

    let output:JObject = match uvm_core::find_installation(&version) {
        Ok(installation) => env.new_object(class, "(Ljava/lang/String;)V", &[JValue::Object(env.new_string(installation.path().to_string_lossy()).unwrap().into())]).unwrap(),
        Err(_) => JObject::null()
    };
    output.into_inner()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_UnityVersionManager_installEditor(env: JNIEnv, _class: JClass, version: JString, destination: JObject, iOS: jboolean, android: jboolean, webGl: jboolean) -> jboolean {
    // //setup_log();
    let version:String = env.get_string(version).expect("Couldn't get java string!").into();
    let version = Version::from_str(&version).unwrap();

    let destination:JString = env.call_method(destination, "getPath", "()Ljava/lang/String;", &[]).unwrap().l().unwrap().into();
    let destination:String = env.get_string(destination).expect("Couldn't get java string!").into();
    let destination = Path::new(&destination).to_path_buf();

    let mut variants: HashSet<InstallVariant> = HashSet::with_capacity(3);
    if android != 0 {
        variants.insert(InstallVariant::Android);
    }

    if iOS != 0 {
        variants.insert(InstallVariant::Ios);
    }

    if webGl != 0 {
        variants.insert(InstallVariant::WebGl);
    }

    match install::install(version, Some(destination), Some(variants)) {
        Ok(_) => 1,
        Err(_) => 0
    }
}

// pub extern "system" fn Java_UnityVersionManager_installEditor__Ljava_lang_String_2_3LUnityVersionManager_Component_2(env: JNIEnv, _class: JClass, version: JString) -> jboolean {
//     //setup_log();
//     let version:String = env.get_string(version).expect("Couldn't get java string!").into();
//     let version = Version::from_str(&version).unwrap();
//
//     install::ensure_tap_for_version(&version).ok();
//
//     let installer = install::download_installer(install::InstallVariant::Editor, &version).unwrap();
//     match install::install_editor(&installer, &Path::new(&format!("/Users/larusso/Unity/{}", &version.to_string())).to_path_buf()) {
//         Ok(_) => 1,
//         Err(_) => 0
//     }
// }

fn setup_log() {
    let log_spec_builder = LogSpecification::default(LevelFilter::Debug);
    let log_spec = log_spec_builder.build();
    Logger::with(log_spec)
        //.format(format_logs)
        .start()
        .unwrap();
}
