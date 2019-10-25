use std::collections::HashSet;
use stringreader::StringReader;
use uvm_core::unity::v2::Manifest;
use uvm_core::unity::{Component, Version};
use uvm_install_graph::{InstallGraph, InstallStatus, Walker};
mod fixures;
use itertools::Itertools;

#[test]
fn create_graph_from_manifest() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");

    let _graph = InstallGraph::from(&manifest);
}

#[test]
fn retrieve_nodes() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");

    let graph = InstallGraph::from(&manifest);
    let node_idx = graph.get_node_id(Component::Android).unwrap();

    assert_eq!(*graph.component(node_idx).unwrap(), Component::Android);
}

#[test]
fn install_status() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
    let mut graph = InstallGraph::from(&manifest);

    let node_android = graph.get_node_id(Component::Android).unwrap();
    let node_ios = graph.get_node_id(Component::Ios).unwrap();
    let node_webgl = graph.get_node_id(Component::WebGl).unwrap();
    let node_editor = graph.get_node_id(Component::Editor).unwrap();

    assert_eq!(
        graph.install_status(node_android),
        Some(&InstallStatus::Unknown)
    );
    assert_eq!(
        graph.install_status(node_ios),
        Some(&InstallStatus::Unknown)
    );
    assert_eq!(
        graph.install_status(node_webgl),
        Some(&InstallStatus::Unknown)
    );
    assert_eq!(
        graph.install_status(node_editor),
        Some(&InstallStatus::Unknown)
    );

    let mut installed_set: HashSet<Component> = HashSet::new();
    installed_set.insert(Component::Editor);
    installed_set.insert(Component::Android);
    graph.mark_installed(&installed_set);

    assert_eq!(
        graph.install_status(node_android),
        Some(&InstallStatus::Installed)
    );
    assert_eq!(
        graph.install_status(node_ios),
        Some(&InstallStatus::Missing)
    );
    assert_eq!(
        graph.install_status(node_webgl),
        Some(&InstallStatus::Missing)
    );
    assert_eq!(
        graph.install_status(node_editor),
        Some(&InstallStatus::Installed)
    );

    graph.mark_all_missing();

    assert_eq!(
        graph.install_status(node_android),
        Some(&InstallStatus::Missing)
    );
    assert_eq!(
        graph.install_status(node_ios),
        Some(&InstallStatus::Missing)
    );
    assert_eq!(
        graph.install_status(node_webgl),
        Some(&InstallStatus::Missing)
    );
    assert_eq!(
        graph.install_status(node_editor),
        Some(&InstallStatus::Missing)
    );

    graph.mark_all(InstallStatus::Installed);

    assert_eq!(
        graph.install_status(node_android),
        Some(&InstallStatus::Installed)
    );
    assert_eq!(
        graph.install_status(node_ios),
        Some(&InstallStatus::Installed)
    );
    assert_eq!(
        graph.install_status(node_webgl),
        Some(&InstallStatus::Installed)
    );
    assert_eq!(
        graph.install_status(node_editor),
        Some(&InstallStatus::Installed)
    );
}

#[test]
fn modules_dependencies() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
    let graph = InstallGraph::from(&manifest);

    let node_android = graph.get_node_id(Component::Android).unwrap();
    let node_android_sdk_ndk_tools = graph.get_node_id(Component::AndroidSdkNdkTools).unwrap();
    let node_android_ndk = graph.get_node_id(Component::AndroidNdk).unwrap();
    let node_editor = graph.get_node_id(Component::Editor).unwrap();

    let depended = graph.get_dependend_modules(node_android_ndk);
    let mut iter = depended.iter();

    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidSdkNdkTools, InstallStatus::Unknown),
            node_android_sdk_ndk_tools
        ))
    );

    assert_eq!(
        iter.next(),
        Some(&((Component::Android, InstallStatus::Unknown), node_android))
    );

    assert_eq!(
        iter.next(),
        Some(&((Component::Editor, InstallStatus::Unknown), node_editor))
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn sub_modules() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
    let graph = InstallGraph::from(&manifest);

    let node_android = graph.get_node_id(Component::Android).unwrap();
    let node_open_jdk = graph.get_node_id(Component::AndroidOpenJdk).unwrap();
    let node_android_sdk_ndk_tools = graph.get_node_id(Component::AndroidSdkNdkTools).unwrap();
    let node_android_sdk_platforms = graph.get_node_id(Component::AndroidSdkPlatforms).unwrap();
    let node_android_sdk_platform_tools = graph
        .get_node_id(Component::AndroidSdkPlatformTools)
        .unwrap();
    let node_android_sdk_build_tools = graph.get_node_id(Component::AndroidSdkBuildTools).unwrap();
    let node_android_ndk = graph.get_node_id(Component::AndroidNdk).unwrap();

    let depended = graph.get_sub_modules(node_android);
    let mut iter = depended
        .iter()
        .sorted_by(|((a, _), _), ((b, _), _)| b.to_string().cmp(&a.to_string()));
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidSdkPlatforms, InstallStatus::Unknown),
            node_android_sdk_platforms
        ))
    );
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidSdkPlatformTools, InstallStatus::Unknown),
            node_android_sdk_platform_tools
        ))
    );
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidSdkNdkTools, InstallStatus::Unknown),
            node_android_sdk_ndk_tools
        ))
    );
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidSdkBuildTools, InstallStatus::Unknown),
            node_android_sdk_build_tools
        ))
    );
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidOpenJdk, InstallStatus::Unknown),
            node_open_jdk
        ))
    );
    assert_eq!(
        iter.next(),
        Some(&(
            (Component::AndroidNdk, InstallStatus::Unknown),
            node_android_ndk
        ))
    );
    assert_eq!(iter.next(), None);
}

#[test]
fn topo() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
    let mut graph = InstallGraph::from(&manifest);

    let node_android_ndk = graph.get_node_id(Component::AndroidNdk).unwrap();
    let ndk_dependencies = graph.get_dependend_modules(node_android_ndk);
    let mut keep: HashSet<Component> = ndk_dependencies
        .into_iter()
        .map(|((component, _), _)| component)
        .collect();
    keep.insert(Component::AndroidNdk);
    graph.keep(&keep);
    let mut topo = graph.topo();

    assert_eq!(
        topo.walk_next(graph.context()),
        graph.get_node_id(Component::Editor)
    );
    assert_eq!(
        topo.walk_next(graph.context()),
        graph.get_node_id(Component::Android)
    );
    assert_eq!(
        topo.walk_next(graph.context()),
        graph.get_node_id(Component::AndroidSdkNdkTools)
    );
    assert_eq!(
        topo.walk_next(graph.context()),
        graph.get_node_id(Component::AndroidNdk)
    );
    assert_eq!(topo.walk_next(graph.context()), None);
}

#[test]
fn keep() {
    let ini = fixures::UNITY_2019_INI;
    let reader = StringReader::new(ini);
    let version = Manifest::read_manifest_version(reader).unwrap();
    let reader = StringReader::new(ini);
    let manifest = Manifest::from_reader(&version, reader).expect("a manifest");
    let mut graph = InstallGraph::from(&manifest);

    assert!(graph.node_count() >= 2);

    let mut keep_set: HashSet<Component> = HashSet::new();
    keep_set.insert(Component::Editor);
    keep_set.insert(Component::Android);
    graph.keep(&keep_set);

    assert!(graph.node_count() == 2);
}
