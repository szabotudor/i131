use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
};

use crate::systems::SystemError;

pub trait SystemScheduler {
    fn schedule(
        &self,
        dependency_tree: &HashMap<TypeId, HashSet<TypeId>>,
        num_threads: usize,
    ) -> Result<Vec<Vec<TypeId>>, SystemError>;
}

#[derive(Default)]
pub struct DAGScheduler {}
impl DAGScheduler {
    pub fn new() -> Self {
        Self::default()
    }
}
impl SystemScheduler for DAGScheduler {
    fn schedule(
        &self,
        tree: &HashMap<TypeId, HashSet<TypeId>>,
        num_threads: usize,
    ) -> Result<Vec<Vec<TypeId>>, SystemError> {
        // Computing tiers

        let mut scheduled = HashSet::<TypeId>::new();
        let mut tiers: Vec<HashSet<TypeId>> = Vec::new();

        loop {
            let tier = tree
                .iter()
                .filter(|(sys, deps)| {
                    !scheduled.contains(*sys)
                        && (deps.is_empty() || deps.iter().all(|dep| scheduled.contains(dep)))
                })
                .map(|(sys, _)| *sys)
                .collect::<HashSet<_>>();

            if tier.is_empty() {
                if scheduled.len() < tree.len() {
                    let cause = tree
                        .iter()
                        .filter(|(sys, _)| !scheduled.contains(*sys))
                        .map(|(sys, _)| *sys)
                        .collect::<Vec<_>>();
                    return Err(SystemError::SystemCyclicDependency(cause));
                }
                break;
            }

            scheduled.extend(&tier);
            tiers.push(tier);
        }

        // Computing schedule

        let mut threads = vec![Vec::new(); num_threads];
        let mut thread = 0usize;

        for tier in tiers {
            for sys in tier {
                threads[thread].push(sys);
                thread = (thread + 1) % num_threads;
            }
        }

        Ok(threads)
    }
}
