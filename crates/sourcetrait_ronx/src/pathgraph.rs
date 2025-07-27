use crate::*;

/// Maintains a list of paths and their dependencies to other paths.
#[derive(Debug, Clone)]
pub struct PathGraph {
    dependencies: HashMap<PathBuf, HashSet<PathBuf>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CycleCheckResult {
    NoCycle,
    /// Includes the path that contains the cycle
    Cycle(Vec<PathBuf>),
}

impl PathGraph {
    pub fn new() -> Self {
        PathGraph {
            dependencies: HashMap::new(),
        }
    }
    
    pub fn add_file(&mut self, file: impl AsRef<Path>) {
        let file = file.as_ref().to_path_buf();
        self.dependencies.entry(file).or_insert_with(HashSet::new);
    }

    /// Add a file and its dependencies to the graph
    pub fn add_fileset(&mut self, file: impl AsRef<Path>, deps: impl IntoIterator<Item = PathBuf>) -> Result<()> {
        let file = file.as_ref().to_path_buf();
        let dependencies: Vec<_> = deps.into_iter().collect();
        
        for dependency in &dependencies {
            if self.would_create_cycle(&file, &dependency) {
                let cycle = vec![file.clone(), dependency.clone(), file.clone()];
                return Err(PathGraphError::CircularDependency(cycle));
            }
        }
        
        self.dependencies.entry(file)
            .or_insert_with(HashSet::new)
            .extend(dependencies);
        
        Ok(())
    }
    /// Add dependency with immediate cycle check
    pub fn add_dependency(
        &mut self,
        file: impl AsRef<Path>,
        dependency: impl AsRef<Path>,
    ) -> Result<()> {
        let file = file.as_ref().to_path_buf();
        let dependency = dependency.as_ref().to_path_buf();
        
        if self.would_create_cycle(&file, &dependency) {
            let cycle = vec![file.clone(), dependency.clone(), file.clone()];
            return Err(PathGraphError::CircularDependency(cycle));
        }
        
        self.dependencies
            .entry(file)
            .or_insert_with(HashSet::new)
            .insert(dependency);
        
        Ok(())
    }

    /// Check if there are any circular dependencies
    pub fn check_for_cycles(&self) -> CycleCheckResult {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for file in self.dependencies.keys() {
            if !visited.contains(file) {
                if let Some(cycle) = self.dfs_find_cycle(file, &mut visited, &mut rec_stack, &mut path) {
                    return CycleCheckResult::Cycle(cycle);
                }
            }
        }

        CycleCheckResult::NoCycle
    }

    /// Depth-first-search for cycles
    fn dfs_find_cycle(
        &self,
        node: &PathBuf,
        visited: &mut HashSet<PathBuf>,
        rec_stack: &mut HashSet<PathBuf>,
        path: &mut Vec<PathBuf>,
    ) -> Option<Vec<PathBuf>> {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let Some(deps) = self.dependencies.get(node) {
            for dep in deps {
                if !visited.contains(dep) {
                    if let Some(cycle) = self.dfs_find_cycle(dep, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    // found a cycle. build the cycle path
                    let cycle_start = path.iter().position(|p| p == dep).unwrap();
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(dep.clone()); // add the dependency to close the cycle
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }

    /// Get all dependencies of a file (direct dependencies only)
    pub fn get_dependencies(&self, file: impl AsRef<Path>) -> Option<&HashSet<PathBuf>> {
        self.dependencies.get(file.as_ref())
    }

    /// Get all transitive dependencies of a file
    pub fn get_all_dependencies(&self, file: impl AsRef<Path>) -> HashSet<PathBuf> {
        let mut all_deps = HashSet::new();
        let mut to_visit = vec![file.as_ref().to_path_buf()];
        let mut visited = HashSet::new();

        while let Some(current) = to_visit.pop() {
            if visited.insert(current.clone()) {
                if let Some(deps) = self.dependencies.get(&current) {
                    all_deps.extend(deps.clone());
                    to_visit.extend(deps.clone());
                }
            }
        }

        all_deps
    }

    /// Check if adding a new dependency would create a cycle
    pub fn would_create_cycle(&self, file: impl AsRef<Path>, new_dep: impl AsRef<Path>) -> bool {
        let file = file.as_ref();
        let new_dep = new_dep.as_ref();
        let deps_of_new_dep = self.get_all_dependencies(new_dep);
        deps_of_new_dep.contains(file)
    }

    /// Get a topological ordering of files (if no cycles exist)
    pub fn topological_sort(&self) -> std::result::Result<Vec<PathBuf>, Vec<PathBuf>> {
        if let CycleCheckResult::Cycle(cycle) = self.check_for_cycles() {
            return Err(cycle);
        }

        let mut in_degree = HashMap::new();
        let mut result = Vec::new();

        // initialize in-degrees
        for file in self.dependencies.keys() {
            in_degree.entry(file.clone()).or_insert(0);
            if let Some(deps) = self.dependencies.get(file) {
                for dep in deps {
                    *in_degree.entry(dep.clone()).or_insert(0) += 1;
                }
            }
        }

        // find all nodes with in-degree 0
        let mut queue: Vec<_> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(file, _)| file.clone())
            .collect();

        while let Some(file) = queue.pop() {
            result.push(file.clone());

            if let Some(deps) = self.dependencies.get(&file) {
                for dep in deps {
                    let deg = in_degree.get_mut(dep).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push(dep.clone());
                    }
                }
            }
        }

        Ok(result)
    }

    /// Clear all tracked dependencies
    pub fn clear(&mut self) {
        self.dependencies.clear();
    }

    /// Get the number of files being tracked
    pub fn file_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Debug print the graph
    pub fn debug_print(&self) {
        for (file, deps) in &self.dependencies {
            println!("{:?} depends on:", file);
            for dep in deps {
                println!("  -> {:?}", dep);
            }
        }
    }
}

impl Default for PathGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let mut graph = PathGraph::new();
        
        graph.add_fileset("main.rs", vec![
            PathBuf::from("lib.rs"),
            PathBuf::from("utils.rs"),
        ]).unwrap();
        graph.add_fileset("lib.rs", vec![PathBuf::from("utils.rs")]).unwrap();
        graph.add_fileset("utils.rs", vec![]).unwrap();

        assert_eq!(graph.check_for_cycles(), CycleCheckResult::NoCycle);
    }

    #[test]
    fn test_simple_cycle() {
        let mut graph = PathGraph::new();
        
        graph.add_fileset("a.rs", vec![PathBuf::from("b.rs")]).unwrap();
        graph.add_fileset("b.rs", vec![PathBuf::from("c.rs")]).unwrap();
        
        assert!(graph.add_fileset("c.rs", vec![PathBuf::from("a.rs")]).is_err());
    }

    #[test]
    fn test_self_cycle() {
        let mut graph = PathGraph::new();
        
        graph.add_fileset("self.rs", vec![PathBuf::from("self.rs")]).unwrap();

        match graph.check_for_cycles() {
            CycleCheckResult::Cycle(path) => {
                assert!(path.contains(&PathBuf::from("self.rs")));
            }
            CycleCheckResult::NoCycle => panic!("Expected a cycle"),
        }
    }

    #[test]
    fn test_would_create_cycle() {
        let mut graph = PathGraph::new();
        
        graph.add_fileset("a.rs", vec![PathBuf::from("b.rs")]).unwrap();
        graph.add_fileset("b.rs", vec![PathBuf::from("c.rs")]).unwrap();
        
        // Adding c.rs -> a.rs would create a cycle
        assert!(graph.would_create_cycle("c.rs", "a.rs"));
        
        // Adding c.rs -> d.rs would not create a cycle
        assert!(!graph.would_create_cycle("c.rs", "d.rs"));
    }

    #[test]
    fn test_topological_sort() {
        let mut graph = PathGraph::new();
        
        graph.add_fileset("main.rs", vec![
            PathBuf::from("lib.rs"),
            PathBuf::from("utils.rs"),
        ]).unwrap();
        graph.add_fileset("lib.rs", vec![PathBuf::from("core.rs")]).unwrap();
        graph.add_fileset("utils.rs", vec![PathBuf::from("core.rs")]).unwrap();
        graph.add_fileset("core.rs", vec![]).unwrap();

        let sorted = graph.topological_sort().unwrap();
        
        // main.rs should come before its dependencies
        let main_idx = sorted.iter().position(|p| p == Path::new("main.rs")).unwrap();
        let lib_idx = sorted.iter().position(|p| p == Path::new("lib.rs")).unwrap();
        let utils_idx = sorted.iter().position(|p| p == Path::new("utils.rs")).unwrap();
        
        assert!(main_idx < lib_idx);
        assert!(main_idx < utils_idx);
    }
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PathGraphError {
    #[error("Circular dependency detected: {}", format_cycle(.0))]
    CircularDependency(Vec<PathBuf>),
    
    #[error("File not found in graph: {0}")]
    FileNotFound(PathBuf),
}

pub type Result<T> = std::result::Result<T, PathGraphError>;

fn format_cycle(cycle: &[PathBuf]) -> String {
    cycle.iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(" -> ")
}

