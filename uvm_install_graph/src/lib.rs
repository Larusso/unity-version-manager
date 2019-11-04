//! **uvm_install_graph** is a helper library to visualize and traverse a unity installation manifest.
use petgraph::visit::NodeIndexable;
pub use daggy::petgraph;
use daggy::petgraph::graph::DefaultIx;
use daggy::petgraph::visit::Topo;
use daggy::Dag;
use daggy::NodeIndex;
pub use daggy::Walker;
use itertools::Itertools;
use std::collections::HashSet;
use std::fmt;
use uvm_core::unity::{Manifest, Component, Version};

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

type DagNode = (Component, InstallStatus);
type DagEdge = ();
type ModulesDag = Dag<DagNode, DagEdge>;

/// A simple [directed acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph)
/// representation of a unity version and it's modules. This graph allows the traversal of to
/// be installed modules in a topological order.
/// It is also possible to query submodule and or dependencies for a given module.
#[derive(Debug)]
pub struct InstallGraph<'a> {
    manifest: &'a Manifest<'a>,
    dag: ModulesDag,
}

impl<'a> From<&'a Manifest<'a>> for InstallGraph<'a> {
    fn from(manifest: &'a Manifest<'a>) -> Self {
        Self::from(manifest)
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

    pub fn keep(&mut self, modules: &HashSet<Component>) {
        self.dag = self.dag.filter_map(
            |_n, (module, install_status)| {
                if modules.contains(&module) {
                    Some((*module, *install_status))
                } else {
                    None
                }
            },
            |_, _| Some(()),
        );
    }

    /// Builds a Dag representation of the given **Manifest**.
    pub fn from(manifest: &'a Manifest<'a>) -> Self {
        let dag = Self::build_dag(&manifest);
        InstallGraph {
            manifest,
            dag
        }
    }

    /// Creates a `Topo` traversal struct from the current graph to walk the graph in topological order.
    pub fn topo(&self) -> Topo<NodeIndex, fixedbitset::FixedBitSet> {
        Topo::new(&self.dag)
    }

    pub fn version(&self) -> &Version {
        self.manifest.version()
    }

    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Mark all nodes with the given `InstallStatus`.
    pub fn mark_all(&mut self, install_status: InstallStatus) {
        self.dag = self
            .dag
            .map(|_, (module, _)| (*module, install_status), |_, e| (*e));
    }

    /// Mark all nodes as `InstallStatus::Missing`.
    pub fn mark_all_missing(&mut self) {
        self.mark_all(InstallStatus::Missing)
    }

    /// Mark all nodes contained in `modules` as `InstallStatus::Installed`.
    /// If the node is not found in `modules` the node gets marked as `InstallStatus::Missing`.
    pub fn mark_installed(&mut self, modules: &HashSet<Component>) {
        self.dag = self.dag.filter_map(
            |_n, (module, _)| {
                if modules.contains(&module) {
                    Some((*module, InstallStatus::Installed))
                } else {
                    Some((*module, InstallStatus::Missing))
                }
            },
            |_, _| Some(()),
        );
    }

    /// Returns the component for the given node index.
    pub fn component(&self, node: NodeIndex) -> Option<&Component> {
        self.dag.node_weight(node).map(|(component, _)| component)
    }

    pub fn install_status(&self, node: NodeIndex) -> Option<&InstallStatus> {
        self.dag.node_weight(node).map(|(_, status)| status)
    }

    pub fn get_node_id(&self, component: Component) -> Option<NodeIndex> {
        self.dag.raw_nodes().iter().enumerate().find(|(_,node)| {
            let (c, _) = node.weight;
            c == component
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
    pub fn get_sub_modules(&self, node: NodeIndex) -> Vec<(DagNode, NodeIndex)> {
        let mut modules = Vec::new();
        for (_, n) in self.dag.children(node).iter(&self.dag) {
            modules.push((*self.dag.node_weight(n).unwrap(), n));
            modules.append(&mut self.get_sub_modules(n));
        }
        modules
    }

    pub fn context(&self) -> &ModulesDag {
        &self.dag
    }

    fn build_dag(manifest: &Manifest) -> ModulesDag {
        let mut dag = Dag::new();

        let editor = dag.add_node((Component::Editor, InstallStatus::default()));

        for (c, m1) in manifest
            .iter()
            .sorted_by(|(a, _), (b, _)| b.to_string().cmp(&a.to_string()))
        {
            if *c != Component::Editor && m1.sync.is_none() {
                let (_, n) = dag.add_child(editor, (), (*c, InstallStatus::default()));
                Self::find_sync(&mut dag, &manifest, *c, n, 1);
            }
        }
        dag
    }

    fn find_sync(
        dag: &mut ModulesDag,
        manifest: &Manifest,
        parent: Component,
        n: NodeIndex<DefaultIx>,
        depth: u64,
    ) {
        for (c, m) in manifest.iter() {
            match m.sync {
                Some(id) if id == parent => {
                    let (_, n) = dag.add_child(n, (), (*c, InstallStatus::default()));
                    Self::find_sync(dag, manifest, *c, n, depth + 1);
                }
                _ => (),
            };
        }
    }
}
