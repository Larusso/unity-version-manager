use super::*;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use unity::hub::paths;

pub fn all_versions() -> io::Result<impl Iterator<Item = Version>> {
    let url = reqwest::Url::parse("https://unity-versions-service.herokuapp.com/").unwrap();
    let url = url
        .join("versions/")
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to create URL"))?;

    let versions: BTreeMap<Version, String> = reqwest::get(url)
        .and_then(|mut response| response.json())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to load content"))?;

    Ok(versions.into_iter().map(|(version, _)| version))
}

fn hash_from_service(version: &Version) -> io::Result<String> {
    debug!("fetch hash for version {} from versions service", version);
    let url = reqwest::Url::parse("https://unity-versions-service.herokuapp.com/").unwrap();
    let url = url
        .join("versions/")
        .and_then(|url| url.join(&format!("{}/", version)))
        .and_then(|url| url.join("hash"))
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to create URL"))?;

    let hash: String = reqwest::get(url)
        .and_then(|mut response| response.json())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Unable to load content"))?;
    debug!("loaded hash {}", &hash);
    Ok(hash)
}

fn hash_from_cache(version: &Version) -> io::Result<String> {
    use std::io::Read;
    debug!("fetch hash for version {} from cache", version);
    let path = cache_file(version)?;
    let mut file = fs::File::open(&path)?;
    let mut hash = String::new();
    file.read_to_string(&mut hash)?;
    Ok(hash)
}

fn cache_file(version: &Version) -> io::Result<PathBuf> {
    paths::hash_cache_dir()
        .map(|p| p.join(&format!("{}.hash", version)))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Unable to fetch cache dir"))
}

fn cache_hash<H: AsRef<str>>(hash: H, version: &Version) -> io::Result<()> {
    use std::fs::DirBuilder;
    use std::io::Write;
    let hash = hash.as_ref();
    let path = cache_file(version)?;
    DirBuilder::new()
        .recursive(true)
        .create(&path.parent().unwrap())?;
    let mut file = fs::File::create(&path)?;
    file.write_all(hash.as_bytes())
}

pub fn hash_for_version(version: &Version) -> io::Result<String> {
    let versions: BTreeMap<Version, String> = serde_yaml::from_str(VERSIONS)
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Unable to load versions.yml"))?;

    versions
        .get(version)
        .cloned()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Unable to find hash for version"))
        .or_else(|_| {
            debug!("hash for version {} not available in memory", version);
            hash_from_cache(version)
        }).or_else(|_| {
            debug!("hash for version {} not available in cache", version);
            hash_from_service(version).and_then(|hash| {
                cache_hash(&hash, version).unwrap_or_else(|err| {
                    warn!("unable to cache hash {} for version {}", &hash, version);
                    warn!("error: {}", err.description());
                });
                Ok(hash)
            })
        })
}

const VERSIONS: &str = "
5.4.5f1: 68943b6c8c42
5.5.0f3: 38b4efef76f0
5.5.1f1: 88d00a7498cd
5.5.2f1: 3829d7f588f3
5.5.3f1: 4d2f809fd6f3
5.5.4f1: 8ffd0efd98b1
5.5.5f1: d875e6967482
5.6.0f3: 497a0f351392
5.6.1f1: 2860b30f0b54
5.6.2f1: a2913c821e27
5.6.3f1: d3101c3b8468
5.6.4f1: ac7086b8d112
2017.1.0f1: 4d7fe18a2f34
2017.1.0f2: 66e9e4bfc850
2017.1.0f3: 472613c02cf7
2017.1.1f1: 5d30cf096e79
2017.1.2f1: cc85bf6a8a04
2017.1.3f1: 574eeb502d14
2017.1.4f1: 9fd71167a288
2017.1.5f1: 9758a36cfaa6
2017.2.0f1: 35e55a2a85de
2017.2.0f2: 472de62575d5
2017.2.0f3: 46dda1414e51
2017.2.1f1: 94bf3f9e6b5e
2017.2.2f1: 1f4e0f9b6a50
2017.2.3f1: 372229934efd
2017.2.4f1: f1557d1f61fd
2017.3.0f1: b84f5794ed91
2017.3.0f2: d3a5469e8c44
2017.3.0f3: a9f86dcd79df
2017.3.1f1: fc1d3344e6ea
2017.4.0f1: b5bd171ee9ba
2017.4.10f1: f2cce2a5991f
2017.4.11f1: 8c6b8ef6d111
2017.4.12f1: b582b87345b1
2017.4.1f1: 9231f953d9d3
2017.4.2f2: 52d9cb89b362
2017.4.3f1: 21ae32b5a9cb
2017.4.4f1: 645c9050ba4d
2017.4.5f1: 89d1db9cb682
2017.4.6f1: c24f30193bac
2017.4.7f1: de9eb5ca33c5
2017.4.8f1: 5ab7f4878ef1
2017.4.9f1: 6d84dfc57ccf
2018.1.0f1: 21784bcc13fa
2018.1.0f2: d4d99f31acba
2018.1.1f1: b8cbb5de9840
2018.1.2f1: a46d718d282d
2018.1.3f1: a53ad04f7c7f
2018.1.4f1: 1a308f4ebef1
2018.1.5f1: 732dbf75922d
2018.1.6f1: 57cc34175ccf
2018.1.7f1: 4cb482063d12
2018.1.8f1: 26051d4de9e9
2018.1.9f1: 24bbd83e8b9e
2018.1.9f2: a6cc294b73ee
2018.2.0f1: 51acc5a75f1e
2018.2.0f2: '787658998520'
2018.2.10f1: 674aa5a67ed5
2018.2.11f1: 38bd7dec5000
2018.2.1f1: 1a9968d9f99c
2018.2.2f1: c18cef34cbcd
2018.2.3f1: 1431a7d2ced7
2018.2.4f1: cb262d9ddeaf
2018.2.5f1: 3071d1717b71
2018.2.6f1: c591d9a97a0b
2018.2.7f1: 4ebd28dd9664
2018.2.8f1: ae1180820377
2018.2.9f1: '2207421190e9'
2017.4.13f1: 6902ad48015d
2018.2.12f1: 0a46ddfcfad4
2018.2.13f1: 83fbdcd35118
2017.4.14f1: b28150134d55
2018.2.14f1: 3262fb3b0716
5.5.2p1: 9360c5517afe
5.5.3p2: f15b2772e4d0
5.6.0p2: bbd5ca01a0ea
5.6.0p3: f8dcc233883f
5.6.1p1: 74c1f4917542
5.6.3p3: 88d4ddf6344a
5.6.3p4: fbe8bd37d7fa
5.6.4p1: e67c4b7007d5
2017.1.0p1: 2f459b492f3c
2017.1.0p2: 31954117ff6c
2017.1.0p3: 0f0686ba7d25
2017.1.0p5: de463fc61bac
2017.1.1p1: 007fc09e806c
2017.1.1p2: b8e3f2d6c409
2017.1.1p3: 929150d2fa14
2017.1.1p4: 4b0ddcd3f6ad
2017.1.2p1: c2ed782bb21e
2017.1.2p2: eba6bfec1bb2
2017.1.2p3: 249a06fbaf10
2017.1.2p4: d597d0924185
2017.1.3p1: 02d73f71d3bd
2017.1.3p2: 744dab055778
2017.1.3p3: fc055e6cd68b
2017.1.3p4: 918e58443b8e
2017.1.4p1: '644977348e46'
2017.1.4p2: 490bad3999ec
2017.2.0p1: 24fd82ce573a
2017.2.0p2: dbc2eb12ac98
2017.2.0p3: 40117ac43b95
2017.2.0p4: 0c3a6a294e34
2017.2.1p1: edf5bdf50eb0
2017.2.1p2: 1dc514532f08
2017.2.1p3: 273860332f50
2017.2.1p4: 1992a1ed2d78
2017.2.2p1: 31794ac12ad1
2017.2.2p2: 32bc645ba6f6
2017.2.2p3: 7706f9f606ca
2017.2.2p4: a30add86d148
2017.2.3p1: b4bae9093154
2017.2.3p2: 21333da13d02
2017.2.3p3: 726d0db4eeac
2017.2.3p4: 2f2d0e6b4eb5
2017.3.0p1: 4596dd67072f
2017.3.0p2: b91e4c5f54ad
2017.3.0p3: bfcbae508940
2017.3.0p4: 25a5860ad58d
2017.3.1p1: 6c5ba423732e
2017.3.1p2: fd9fec26f216
2017.3.1p3: a66397957d3b
2017.3.1p4: 7f25373c3e03
2017.1.0b10: 94eaa37e21dd
2017.1.0b1: a29fc4a7eb25
2017.1.0b2: 5e138e18bf82
2017.1.0b3: 9393889e4fe6
2017.1.0b4: ab0150af3e1e
2017.1.0b5: e2f219641e2c
2017.1.0b6: 38ec4e48ade7
2017.1.0b7: 8a1ad67dc191
2017.1.0b8: 17011ab1b2e1
2017.1.0b9: a1e6a9071015
2017.2.0b10: 6c4d42ddd191
2017.2.0b11: 614980c52f17
2017.2.0b1: 1a1b26354326
2017.2.0b2: a9976befbe0f
2017.2.0b3: 4c2ac554540c
2017.2.0b4: 0c51d4e28d7f
2017.2.0b5: 9b4733af38e7
2017.2.0b6: b6bf3c071fe7
2017.2.0b7: 4bc201a72e4a
2017.2.0b8: c5fc3ca9cbbf
2017.2.0b9: 95ec3a4d5d9d
2017.3.0b10: d1367129888f
2017.3.0b11: 8e840c60cd77
2017.3.0b1: bc2668834c45
2017.3.0b2: ec6e8c8c3015
2017.3.0b3: 28dc7ce05bb9
2017.3.0b4: 4c1b6e48c9c9
2017.3.0b5: af76f56822bf
2017.3.0b6: 57ec95547059
2017.3.0b7: 93b5ce6f4b0f
2017.3.0b8: d2b3b1ff7201
2017.3.0b9: e18fe9bb4e54
2018.1.0b10: 4ec9a3104331
2018.1.0b11: c5bf62a40d4b
2018.1.0b12: b4ca90bad6b1
2018.1.0b13: 43de91b8ac41
2018.1.0b1: bcd94551ef32
2018.1.0b2: 79c3bdce0980
2018.1.0b3: e1ef60e69006
2018.1.0b4: 003615bcffde
2018.1.0b5: a48da4f646ae
2018.1.0b6: 2c4679632cfb
2018.1.0b7: cfaabe8e4f18
2018.1.0b8: 0b50224845b9
2018.1.0b9: 36a41ae63f8e
2018.2.0b10: 4bc57476174c
2018.2.0b11: 912020d71ebf
2018.2.0b2: 96999d86066c
2018.2.0b3: 0a6b93065060
2018.2.0b4: a3564b9ba417
2018.2.0b5: 35351042bf9d
2018.2.0b6: ac34ff94dd0f
2018.2.0b7: 8ce15a37a3ae
2018.2.0b8: fed204371f5a
2018.2.0b9: 3b5ad740cdc8
2018.3.0b1: 3f0ac31c6d6f
2018.3.0b2: 21e0e8a5466d
2018.3.0b3: cc0086a8e10c
2018.3.0b4: 44012bad7987
2018.3.0b5: 01088ee0a3a8
2018.3.0b6: f5aefbeed0ac
2018.3.0b7: af029f4527e0
2018.3.0b8: fa755def4b97
2019.1.0a7: 4474d51790a5";
