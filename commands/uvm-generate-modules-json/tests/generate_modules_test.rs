use serde::Serialize;
use uvm_core::unity::{Version, Manifest, Modules, Module};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::path::Path;
use stringreader::StringReader;
mod fixures;

fn save_json_output<P: AsRef<Path>, T: ?Sized + Serialize>(dir:P, file_name: &str, value:&T) -> std::io::Result<()> {
    use std::fs::OpenOptions;
    use std::fs::DirBuilder;
    use std::io::{Error,ErrorKind};

    let base = dir.as_ref();
    DirBuilder::new().recursive(true).create(base)?;
    OpenOptions::new().write(true).create(true).open(base.join(file_name))
        .and_then(|f| serde_json::to_writer_pretty(f, value).map_err(|err| Error::new(ErrorKind::Other, err)))
}

macro_rules! generate_modules_json {
    ($($id:ident, $version:expr, $fixture_name:ident),*) => {
        $(
            #[test]
            fn $id() {
                let version = $version;
                let reader = StringReader::new(fixures::manifest::$fixture_name);
                let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
                let mut a:Modules = manifest.into_modules();
                let mut b:Modules = serde_json::from_str(fixures::module::$fixture_name).expect("a deserialized module");

                a.sort();
                b.sort();

                let name = format!("outputs/{}", stringify!($id));
                let base = Path::new(&name);

                save_json_output(&base, &format!("{}_l.json", &version), &a).expect("a saved output file");
                save_json_output(&base, &format!("{}_r.json", &version), &b).expect("a saved output file");

                let hash_set_a:HashSet<Module> = HashSet::from_iter(a.into_iter());
                let hash_set_b:HashSet<Module> = HashSet::from_iter(b.into_iter());

                let diff = hash_set_a.difference(&hash_set_b);

                assert_eq!(hash_set_a.len(), hash_set_b.len(), "has same length");
                assert_eq!(diff.count(), 0, "has no differences");
            }
        )*
    };
}

#[cfg(not(target_os = "linux"))]
generate_modules_json![
    generates_modules_2022_2, Version::f(2022, 2, 6, 1), UNITY_2022_2_6_F_1,
    generates_modules_2022_1, Version::a(2022, 1, 0, 13), UNITY_2022_1_0_A_13,
    generates_modules_2021_1, Version::f(2021, 1, 28, 1), UNITY_2021_1_28_F_1,
    generates_modules_2021_2, Version::f(2021, 2, 2, 1), UNITY_2021_2_2_F_1,
    generates_modules_2020_1, Version::f(2020, 1, 17, 1), UNITY_2020_1_17_F_1,
    generates_modules_2020_2, Version::f(2020, 2, 7, 1), UNITY_2020_2_7_F_1,
    generates_modules_2020_3, Version::f(2020, 3, 22, 1), UNITY_2020_3_22_F_1,
    generates_modules_2019_1, Version::f(2019, 1, 14, 1), UNITY_2019_1_14_F_1,
    generates_modules_2019_2, Version::f(2019, 2, 21, 1), UNITY_2019_2_21_F_1,
    generates_modules_2019_3, Version::f(2019, 3, 15, 1), UNITY_2019_3_15_F_1,
    generates_modules_2019_4, Version::f(2019, 4, 32, 1), UNITY_2019_4_32_F_1,
    generates_modules_2018_4, Version::f(2018, 4, 36, 1), UNITY_2018_4_36_F_1,
    generates_modules_2018_3, Version::f(2018, 3, 14, 1), UNITY_2018_3_14_F_1,
    generates_modules_2018_2, Version::f(2018, 2, 21, 1), UNITY_2018_2_21_F_1,
    generates_modules_2018_1, Version::f(2018, 1, 9, 2), UNITY_2018_1_9_F_2,
    generates_modules_2017_4, Version::f(2017, 4, 40, 1), UNITY_2017_4_40_F_1,
    generates_modules_2017_3, Version::f(2017, 3, 1, 1), UNITY_2017_3_1_F_1,
    generates_modules_2017_2, Version::f(2017, 2, 5, 1), UNITY_2017_2_5_F_1,
    generates_modules_2017_1, Version::f(2017, 1, 5, 1), UNITY_2017_1_5_F_1
];

#[cfg(target_os = "linux")]
generate_modules_json![
    generates_modules_2022_2, Version::f(2022, 2, 6, 1), UNITY_2022_2_6_F_1,
    generates_modules_2022_1, Version::a(2022, 1, 0, 13), UNITY_2022_1_0_A_13,
    generates_modules_2021_1, Version::f(2021, 1, 28, 1), UNITY_2021_1_28_F_1,
    generates_modules_2021_2, Version::f(2021, 2, 2, 1), UNITY_2021_2_2_F_1,
    generates_modules_2020_1, Version::f(2020, 1, 17, 1), UNITY_2020_1_17_F_1,
    generates_modules_2020_2, Version::f(2020, 2, 7, 1), UNITY_2020_2_7_F_1,
    generates_modules_2020_3, Version::f(2020, 3, 22, 1), UNITY_2020_3_22_F_1,
    generates_modules_2019_1, Version::f(2019, 1, 14, 1), UNITY_2019_1_14_F_1,
    generates_modules_2019_2, Version::f(2019, 2, 21, 1), UNITY_2019_2_21_F_1,
    generates_modules_2019_3, Version::f(2019, 3, 15, 1), UNITY_2019_3_15_F_1,
    generates_modules_2019_4, Version::f(2019, 4, 32, 1), UNITY_2019_4_32_F_1,
    generates_modules_2018_4, Version::f(2018, 4, 36, 1), UNITY_2018_4_36_F_1,
    generates_modules_2018_3, Version::f(2018, 3, 14, 1,), UNITY_2018_3_14_F_1,
    generates_modules_2018_2, Version::f(2018, 2, 21, 1,), UNITY_2018_2_21_F_1,
    generates_modules_2018_1, Version::f(2018, 1, 9, 2,), UNITY_2018_1_9_F_2,
    generates_modules_2017_4, Version::f(2017, 4, 40, 1), UNITY_2017_4_40_F_1
];
