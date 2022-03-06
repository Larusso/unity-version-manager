use crate::unity::Version;
use self::Localization::*;
use std::collections::HashSet;
use std::str::FromStr;

pub mod error {
    use thiserror::Error;

    #[derive(Error, Debug)]
    pub enum Error {
        #[error("unknown locale {0}")]
        Unknown(String)
    }
}

/// add the localization generic information to the config
/// ja and ko are available since 2018.1+
/// zh-cn is available since 2018.2+
/// zh-hant is available since 2019.1+
/// zh-cn changed to zh-hans since 2019.1+
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Deserialize, Serialize)]
pub enum Localization {
    Ja,
    Ko,
    Fr,
    Es,
    ZhCn,
    ZhHant,
    ZhHans,
    Ru
}

impl Localization {

    pub fn locals<V: AsRef<Version>>(version:V) -> impl Iterator<Item=Localization> {
        let mut locales:HashSet<Localization> = HashSet::new();
        let version = version.as_ref();
        // locales.insert(Fr);
        // locales.insert(Es);
        // locales.insert(Ru);
        if *version >= Version::a(2018,1,0,0) {
            locales.insert(Ja);
            locales.insert(Ko);
        }
        if *version >= Version::a(2018,2,0,0) {
            locales.insert(ZhCn);
        }
        if *version >= Version::a(2019,1,0,0) {
            locales.insert(ZhHant);
            locales.insert(ZhHans);
            locales.remove(&ZhCn);
        }
        locales.into_iter()
    }

    pub fn locale(self) -> &'static str {
        match self {
            Ja => "ja",
            Ko => "ko",
            Fr => "fr",
            Es => "es",
            ZhCn => "zh-cn",
            ZhHant => "zh-hant",
            ZhHans => "zh-hans",
            Ru => "ru",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Ja => "日本語",
            Ko => "한국어",
            Fr => "Français",
            Es => "Español",
            ZhCn => "简体中文",
            ZhHant => "繁體中文",
            ZhHans => "简体中文",
            Ru => "русский",
        }
    }
}

impl FromStr for Localization {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ja" => Ok(Ja),
            "ko" => Ok(Ko),
            "fr" => Ok(Fr),
            "es" => Ok(Es),
            "zh-cn" => Ok(ZhCn),
            "zh-hant" => Ok(ZhHant),
            "zh-hans" => Ok(ZhHans),
            "ru" => Ok(Ru),
            x => Err(error::Error::Unknown(x.to_string()))
        }
    }
}
