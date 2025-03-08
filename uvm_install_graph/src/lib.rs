//! **uvm_install_graph** is a helper library to visualize and traverse a api installation manifest.
use petgraph::visit::NodeIndexable;
pub use daggy::petgraph;
use daggy::petgraph::graph::DefaultIx;
use daggy::petgraph::visit::Topo;
use daggy::Dag;
use daggy::NodeIndex;
pub use daggy::Walker;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;
use uvm_live_platform::{Download, Release, Module};
use unity_version::Version;

/// `InstallStatus` is a marker enum to mark nodes in the `InstallGraph` based on the known
/// installation status. The default is **Unknown**.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallStatus {
    Unknown,
    Missing,
    Installed,
}

impl Default for InstallStatus {
    fn default() -> InstallStatus {
        InstallStatus::Unknown
    }
}

impl fmt::Display for InstallStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallStatus::Missing => write!(f, "missing"),
            InstallStatus::Installed => write!(f, "installed"),
            _ => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum UnityComponent<'a> {
    Editor(&'a Download),
    Module(&'a Module)
}

impl Display for UnityComponent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UnityComponent::Editor(_) => {
                write!(f, "Editor")
            }
            UnityComponent::Module(module) => {
                write!(f, "{} - {}", module.id(), module.description())
            }
        }
    }
}

type DagNode<'a> = (UnityComponent<'a>, InstallStatus);
type DagEdge = ();
type ModulesDag<'a> = Dag<DagNode<'a>, DagEdge>;

/// A simple [directed acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph)
/// representation of a api version and it's modules. This graph allows the traversal of to
/// be installed modules in a topological order.
/// It is also possible to query submodule and or dependencies for a given module.
#[derive(Debug)]
pub struct InstallGraph<'a> {
    release: &'a Release,
    dag: ModulesDag<'a>,
}

impl<'a> From<&'a Release> for InstallGraph<'a> {
    fn from(release: &'a Release) -> Self {
        Self::from(release)
    }
}

// use std::ops::Deref;
//
// impl<'a> Deref for InstallGraph<'a> {
//     type Target = ModulesDag;
//
//     fn deref(&self) -> &Self::Target {
//         &self.dag
//     }
// }

impl<'a> InstallGraph<'a> {

    /// The total number of nodes in the Dag.
    pub fn node_count(&self) -> usize {
        self.dag.node_count()
    }

    pub fn keep(&mut self, modules: &HashSet<String>) {
        self.dag = self.dag.filter_map(
            |_n, (c, install_status)| {
                match c {
                    UnityComponent::Editor(_) if modules.contains("Unity") => Some((*c, *install_status)),
                    UnityComponent::Module(module) if modules.contains(module.id()) => Some((*c, *install_status)),
                    _ => None,
                }
            },
            |_, _| Some(()),
        );
    }

    /// Builds a Dag representation of the given **Manifest**.
    pub fn from(release: &'a Release) -> Self {
        let dag = Dag::new();
        let mut graph = InstallGraph {
            release,
            dag
        };
        graph.setup_graph();
        graph
    }

    fn setup_graph(&mut self) {
        let release = &self.release;
        for download in &release.downloads {
            let d = self
                .dag
                .add_node((UnityComponent::Editor(&download), InstallStatus::default()));

            for m in &download.modules {
                let module = self
                    .dag
                    .add_child(d, (), (UnityComponent::Module(&m), InstallStatus::default()));
                for sub in m.sub_modules() {
                    let subModule = self.dag
                        .add_child(module.1, (), (UnityComponent::Module(&sub), InstallStatus::default()));
                    for sub2 in sub.sub_modules() {
                        self.dag.add_child(subModule.1, (), (UnityComponent::Module(&sub2), InstallStatus::default()));
                    }
                }
            }
        }
    }

    /// Creates a `Topo` traversal struct from the current graph to walk the graph in topological order.
    pub fn topo(&self) -> Topo<NodeIndex, fixedbitset::FixedBitSet> {
        Topo::new(&self.dag)
    }

    pub fn version(&self) -> &String {
        &self.release.version
    }

    pub fn release(&self) -> &Release {
        &self.release
    }

    /// Mark all nodes with the given `InstallStatus`.
    pub fn mark_all(&mut self, install_status: InstallStatus) {
        self.dag = self
            .dag
            .map(|_, (c, _)| (*c, install_status), |_, e| (*e));
    }

    /// Mark all nodes as `InstallStatus::Missing`.
    pub fn mark_all_missing(&mut self) {
        self.mark_all(InstallStatus::Missing)
    }

    /// Mark all nodes contained in `components` as `InstallStatus::Installed`.
    /// If the node is not found in `components` the node gets marked as `InstallStatus::Missing`.
    pub fn mark_installed(&mut self, components: &HashSet<String>) {
        self.dag = self.dag.filter_map(
            |_n, (c, _)| {
                match c {
                    UnityComponent::Editor(_) if components.contains("Unity") => Some((*c, InstallStatus::Installed)),
                    UnityComponent::Module(module) if components.contains(module.id()) => Some((*c, InstallStatus::Installed)),
                    _ => Some((*c, InstallStatus::Missing)),
                }
            },
            |_, _| Some(()),
        );
    }

    /// Returns the component for the given node index.
    pub fn component(&self, node: NodeIndex) -> Option<UnityComponent> {
        self.dag.node_weight(node).map(|(component, _)| *component)
    }

    pub fn install_status(&self, node: NodeIndex) -> Option<&InstallStatus> {
        self.dag.node_weight(node).map(|(_, status)| status)
    }

    pub fn get_node_id(&self, component: &str) -> Option<NodeIndex> {
        self.dag.raw_nodes().iter().enumerate().find(|(_, node)| {
            let (c, _) = &node.weight;
            match c {
                UnityComponent::Editor(_) if component == "Unity" => true,
                UnityComponent::Module(module) => module.id() == component,
                _ => false,
            }
        }).map(|(index,_)| self.dag.from_index(index))
    }

    pub fn depth(&self, node: NodeIndex) -> usize {
        self.dag
            .recursive_walk(node, |g, n| g.parents(n).walk_next(g))
            .iter(&self.dag)
            .count()
    }

    /// Returns a **Vec** with all depended modules for the given node.
    pub fn get_dependend_modules(&self, node: NodeIndex) -> Vec<(DagNode, NodeIndex)> {
        self.dag
            .recursive_walk(node, |g, n| g.parents(n).walk_next(g))
            .iter(&self.dag)
            .map(|(_, n)| (*self.dag.node_weight(n).unwrap(), n))
            .collect()
    }

    /// Returns a **Vec** with all submodules for the given node.
    pub fn get_sub_modules(&self, node: NodeIndex) -> Vec<(&DagNode, NodeIndex)> {
        let mut modules = Vec::new();
        for (_, n) in self.dag.children(node).iter(&self.dag) {
            modules.push((self.dag.node_weight(n).unwrap(), n));
            modules.append(&mut self.get_sub_modules(n));
        }
        modules
    }

    pub fn context(&'a self) -> &ModulesDag {
        &self.dag
    }
}
