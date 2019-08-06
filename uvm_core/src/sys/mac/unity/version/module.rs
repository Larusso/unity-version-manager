use crate::unity::{Version};
use crate::unity::Component;

pub struct ModulePart {
    pub component:Component,
    pub name: String,
    pub download_url:String,
    pub version:String,
    pub main:bool,
    pub installed_size:u64,
    pub download_size:u64,
    pub rename_from:Option<String>,
    pub rename_to:Option<String>,
}

pub fn get_android_open_jdk_download_info<V: AsRef<Version>>(_version:V) -> ModulePart {
    ModulePart {
        component: Component::AndroidOpenJdk,
        name: "OpenJDK".to_string(),
        download_url: "http://download.unity3d.com/download_unity/open-jdk/open-jdk-mac-x64/jdk8u172-b11_4be8440cc514099cfe1b50cbc74128f6955cd90fd5afe15ea7be60f832de67b4.zip".to_string(),
        version: "8u172-b11".to_string(),
        main: true,
        installed_size: 72_200_000,
        download_size: 165_000_000,
        rename_from: None,
        rename_to: None
    }
}

pub fn get_android_sdk_ndk_download_info<V: AsRef<Version>>(version:V) -> Vec<ModulePart> {
    let version = version.as_ref();
    let ndk_version = if *version >= Version::a(2019,3,0,0) {
        "r19b"
    } else {
        "r16b"
    };

    vec![
        ModulePart {
            component: Component::AndroidSdkNdkTools,
            name: "Android SDK & NDK Tools".to_string(),
            download_url: "https://dl.google.com/android/repository/sdk-tools-darwin-4333796.zip".to_string(),
            version: "26.1.1".to_string(),
            main: true,
            installed_size: 174_000_000,
            download_size: 148_000_000,
            rename_from: None,
            rename_to: None
        },
        ModulePart {
            component: Component::AndroidSdkPlatformTools,
            name: "Android SDK Platform Tools".to_string(),
            download_url: "https://dl.google.com/android/repository/platform-tools_r28.0.1-darwin.zip".to_string(),
            version: "28.0.1".to_string(),
            main: false,
            installed_size: 15_700_000,
            download_size: 4_550_000,
            rename_from: None,
            rename_to: None
        },
        ModulePart {
            component: Component::AndroidSdkBuildTools,
            name: "Android SDK Build Tools".to_string(),
            download_url: "https://dl.google.com/android/repository/build-tools_r28.0.3-macosx.zip".to_string(),
            version: "28.0.3".to_string(),
            main: false,
            installed_size: 120_000_000,
            download_size: 52_600_000,
            rename_from: Some("{UNITY_PATH}/PlaybackEngines/AndroidPlayer/SDK/build-tools/android-9".to_string()),
            rename_to: Some("{UNITY_PATH}/PlaybackEngines/AndroidPlayer/SDK/build-tools/28.0.3".to_string())
        },
        ModulePart {
            component: Component::AndroidSdkPlatforms,
            name: "Android SDK Platforms".to_string(),
            download_url: "https://dl.google.com/android/repository/platform-28_r06.zip".to_string(),
            version: "28".to_string(),
            main: false,
            installed_size: 121_000_000,
            download_size: 60_600_000,
            rename_from: Some("{UNITY_PATH}/PlaybackEngines/AndroidPlayer/SDK/platforms/android-9".to_string()),
            rename_to: Some("{UNITY_PATH}/PlaybackEngines/AndroidPlayer/SDK/platforms/android-28".to_string())
        },
        ModulePart {
            component: Component::AndroidNdk,
            name: "Android NDK".to_string(),
            download_url: format!("https://dl.google.com/android/repository/android-ndk-{}-darwin-x86_64.zip", ndk_version),
            version: ndk_version.to_string(),
            main: false,
            installed_size: 2_700_000_000,
            download_size: 770_000_000,
            rename_from: Some(format!("{{UNITY_PATH}}/PlaybackEngines/AndroidPlayer/NDK/android-ndk-{}", ndk_version)),
            rename_to: Some("{UNITY_PATH}/PlaybackEngines/AndroidPlayer/NDK".to_string())
        },
    ]
}
