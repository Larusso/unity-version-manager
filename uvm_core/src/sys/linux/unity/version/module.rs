use crate::unity::{Version};
use crate::unity::Component;
use relative_path::{RelativePath, RelativePathBuf};

pub struct ModulePart {
    pub component:Component,
    pub name: String,
    pub download_url:String,
    pub version:String,
    pub main:bool,
    pub installed_size:u64,
    pub download_size:u64,
    pub rename_from:Option<RelativePathBuf>,
    pub rename_to:Option<RelativePathBuf>,
}

pub fn get_android_open_jdk_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    let version = version.as_ref();
    let (version, install_size, download_size) = 
    if *version >= Version::a(2022,1,0,0) {
        ( "8u172-b11", 162_000_000, 73_170_000)
    } else {
        ( "8u172-b11", 162_000_000, 73_170_000)
    };

    ModulePart {
        component: Component::AndroidOpenJdk,
        name: "OpenJDK".to_string(),
        download_url: format!("http://download.unity3d.com/download_unity/open-jdk/open-jdk-linux-x64/jdk{}_4be8440cc514099cfe1b50cbc74128f6955cd90fd5afe15ea7be60f832de67b4.zip", version),
        version: version.to_string(),
        main: true,
        installed_size: install_size,
        download_size: download_size,
        rename_from: None,
        rename_to: None
    }
}

pub fn get_android_ndk_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    let version = version.as_ref();
    let (version, install_size, download_size) = 
    if *version >= Version::a(2021,1,0,0) {
        ( "r21d", 4_341_760_000, 1_126_400_000 )
    } else if *version >= Version::a(2019,3,0,0) {
        ( "r19", 2_690_000_000, 785_000_000 )
    } else {
        ( "r16b", 2_355_200_000, 626_000_000 )
    };
    ModulePart {
        component: Component::AndroidNdk,
        name: "Android NDK".to_string(),
        download_url: format!("https://dl.google.com/android/repository/android-ndk-{}-linux-x86_64.zip", version),
        version: version.to_string(),
        main: false,
        installed_size: install_size,
        download_size: download_size,
        rename_from: Some(RelativePath::new(&format!("{{UNITY_PATH}}/Editor/Data/PlaybackEngines/AndroidPlayer/NDK/android-ndk-{}" , version)).to_relative_path_buf()),
        rename_to: Some(RelativePath::new("{UNITY_PATH}/Editor/Data/PlaybackEngines/AndroidPlayer/NDK").to_relative_path_buf())
    }
}

pub fn get_android_sdk_build_tools_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    let version = version.as_ref();
    let (version, android_version, install_size, download_size) = 
    if *version >= Version::a(2019,4,0,0) {
        ("30.0.2", "11", 140_000_000, 50_200_000 )
    } else {
        ("28.0.3", "9", 120_000_000, 52_600_000 )
    };

    ModulePart {
        component: Component::AndroidSdkBuildTools,
        name: "Android SDK Build Tools".to_string(),
        download_url: format!("https://dl.google.com/android/repository/build-tools_r{}-linux.zip", version),
        version: version.to_string(),
        main: false,
        installed_size: install_size,
        download_size: download_size,
        rename_from: Some(RelativePath::new(&format!("{{UNITY_PATH}}/Editor/Data/PlaybackEngines/AndroidPlayer/SDK/build-tools/android-{}", android_version)).to_relative_path_buf()),
        rename_to: Some(RelativePath::new(&format!("{{UNITY_PATH}}/Editor/Data/PlaybackEngines/AndroidPlayer/SDK/build-tools/{}", version)).to_relative_path_buf())
    }
}

pub fn get_android_sdk_ndk_tools_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    ModulePart {
        component: Component::AndroidSdkNdkTools,
        name: "Android SDK & NDK Tools".to_string(),
        download_url: "https://dl.google.com/android/repository/sdk-tools-linux-4333796.zip".to_string(),
        version: "26.1.1".to_string(),
        main: true,
        installed_size: 174_000_000,
        download_size: 148_000_000,
        rename_from: None,
        rename_to: None
    }
}

pub fn get_android_sdk_platform_tools_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    ModulePart {
        component: Component::AndroidSdkPlatformTools,
        name: "Android SDK Platform Tools".to_string(),
        download_url: "https://dl.google.com/android/repository/platform-tools_r28.0.1-linux.zip".to_string(),
        version: "28.0.1".to_string(),
        main: false,
        installed_size: 15_700_000,
        download_size: 4_550_000,
        rename_from: None,
        rename_to: None
    }
}

pub fn get_android_sdk_platform_download_info<V: AsRef<Version>>(version:V) -> ModulePart {
    let version = version.as_ref();
    let (version, android_version, sdk_version, install_size, download_size) = 
    if *version >= Version::a(2019,4,0,0) {
        ("29_r05", "10", "29", 152_500_000, 78_300_000 )
    } else {
        ("28_r06", "9", "28", 121_000_000, 60_600_000)
    };
    ModulePart {
        component: Component::AndroidSdkPlatforms,
        name: "Android SDK Platforms".to_string(),
        download_url: format!("https://dl.google.com/android/repository/platform-{}.zip", version),
        version: sdk_version.to_string(),
        main: false,
        installed_size: install_size,
        download_size: download_size,
        rename_from: Some(RelativePath::new(&format!("{{UNITY_PATH}}/Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platforms/android-{}", android_version)).to_relative_path_buf()),
        rename_to: Some(RelativePath::new(&format!("{{UNITY_PATH}}/Editor/Data/PlaybackEngines/AndroidPlayer/SDK/platforms/android-{}", sdk_version)).to_relative_path_buf())
    }
}

pub fn get_android_sdk_ndk_download_info<V: AsRef<Version>>(version:V) -> Vec<ModulePart> {
    let version = version.as_ref();
    vec![
        get_android_sdk_ndk_tools_download_info(version),
        get_android_sdk_platform_tools_download_info(version),
        get_android_sdk_build_tools_download_info(version),
        get_android_sdk_platform_download_info(version),
        get_android_ndk_download_info(version)
    ]
}
