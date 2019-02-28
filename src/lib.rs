#[macro_use]
extern crate failure;
use std::collections::HashSet;

#[derive(Fail, Debug)]
pub enum DeptreeError {
    #[fail(display = "Target {} already started, tried to start", _0)]
    AlreadyStarted(String),
    #[fail(display = "Target {} failed, tried to start", _0)]
    StartedFailed(String),
    #[fail(display = "Target {} finished, tried to start", _0)]
    StartedFinished(String),
    #[fail(display = "Target {} not yet started, tried to finish", _0)]
    NotYetStarted(String),
    #[fail(display = "Target {} already finished, tried to finish", _0)]
    AlreadyFinished(String),
    #[fail(display = "Target {} already failed, tried to finish", _0)]
    FinishFailed(String),
    #[fail(display = "Target {} already failed, tried to fail", _0)]
    AlreadyFailed(String),
    #[fail(display = "Target {} not yet started, tried to fail", _0)]
    UnstartedFailed(String)
}

type DeptreeResult<T> = Result<T, DeptreeError>;

#[derive(Eq, PartialEq)]
enum TargetState {
    Unstarted,
    Started,
    Failed,
    Finished
}

#[derive(Hash, Clone, Copy, Eq, PartialEq, Debug)]
pub struct TargetIndex(usize);

type Indices = HashSet<TargetIndex>;
type IndexList = Vec<TargetIndex>;

struct TargetData<Attribs> {
    name : String,
    state : TargetState,
    up : Indices,
    down : Indices,
    attribs : Option<Attribs>
}

impl<Attribs> TargetData<Attribs> {
    fn new(name: String, attribs : Option<Attribs>) -> Self {
        Self { name : name,
               state : TargetState::Unstarted,
               up : Indices::new(),
               down : Indices::new(),
               attribs : attribs
        }
    }
}

type Targets<Attribs> = Vec<TargetData<Attribs>>;

pub struct Deptree<Attribs> {
    targets : Targets<Attribs>,
    roots : Indices,
    leaves : Indices,
    running : usize,
    simple : bool
}

impl<Attribs> Deptree<Attribs> {
    pub fn new() -> Deptree<Attribs> {
        Deptree {
            targets : Targets::new(),
            roots : Indices::new(),
            leaves : Indices::new(),
            running : 0,
            simple : true
        }
    }

    pub fn add_target_attribs<'a>(&'a mut self, name: &str, attribs : Option<Attribs>)
                              -> TargetIndex {
        self.targets.push(TargetData::new(name.to_string(), attribs));
        let index = TargetIndex(self.targets.len() - 1);
        self.roots.insert(index);
        self.leaves.insert(index);
        self.simple = false;
        index
    }
    
    pub fn add_target(&mut self, name: &str) -> TargetIndex {
        self.add_target_attribs(name, None)
    }

    pub fn depend(&mut self, one : TargetIndex, two : TargetIndex) {
        self.targets[one.0].up.insert(two);
        self.targets[two.0].down.insert(one);
        self.roots.remove(&one);
        self.leaves.remove(&two);
        self.simple = false;
    }

    pub fn name(&self, target : TargetIndex) -> &String {
        &self.targets[target.0].name
    }

    pub fn attribs(&self, target : TargetIndex) -> &Option<Attribs> {
        &self.targets[target.0].attribs
    }

    pub fn ready(&self) -> IndexList {
        let mut result = IndexList::with_capacity(self.roots.len());
        for i in self.roots.iter() {
            if self.targets[i.0].state == TargetState::Unstarted {
                result.push(*i);
            }
        }
        result
    }

    pub fn start(&mut self, target : TargetIndex) -> DeptreeResult<()> {
        let mut data = &mut self.targets[target.0];
        match data.state {
            TargetState::Started =>
                Err(DeptreeError::AlreadyStarted(data.name.clone())),
            TargetState::Failed =>
                Err(DeptreeError::StartedFailed(data.name.clone())),
            TargetState::Finished =>
                Err(DeptreeError::StartedFinished(data.name.clone())),
            TargetState::Unstarted => {
                data.state = TargetState::Started;
                self.running += 1;
                Ok(())
            }
        }
    }

    pub fn finish(&mut self, target : TargetIndex) -> DeptreeResult<()> {
        self.simplify();

        let data = &mut self.targets[target.0];
        
        match data.state {
            TargetState::Unstarted =>
                Err(DeptreeError::NotYetStarted(data.name.clone())),
            TargetState::Finished =>
                Err(DeptreeError::AlreadyFinished(data.name.clone())),
            TargetState::Failed =>
                Err(DeptreeError::FinishFailed(data.name.clone())),
            TargetState::Started => {
                data.state = TargetState::Finished;
                for dependent in data.down.iter() {
                    self.roots.insert(*dependent);
                }
                self.running -= 1;
                self.roots.remove(&target);
                Ok(())
            }
        }
    }

    pub fn fail(&mut self, target : TargetIndex) ->DeptreeResult<()> {
        let data = &mut self.targets[target.0];

        match data.state {
            TargetState::Unstarted =>
                Err(DeptreeError::UnstartedFailed(data.name.clone())),
            TargetState::Finished =>
                Err(DeptreeError::FinishFailed(data.name.clone())),
            TargetState::Failed =>
                Err(DeptreeError::AlreadyFailed(data.name.clone())),
            TargetState::Started => {
                data.state = TargetState::Failed;
                self.running -= 1;
                self.roots.remove(&target);
                Ok(())
            }
        }
    }

    pub fn done(&self) -> bool {
        self.running == 0 && self.roots.is_empty()
    }

    pub fn depended_by(&self, target: TargetIndex) -> IndexList {
        let data = &self.targets[target.0];
        
        let mut result = IndexList::with_capacity(data.down.len());
        for dep_by in data.down.iter() {
            result.push(*dep_by);
        }

        result
    }
    
    pub fn depends_on(&self, target: TargetIndex) -> IndexList {
        let data = &self.targets[target.0];
        
        let mut result = IndexList::with_capacity(data.up.len());
        for dep_on in data.up.iter() {
            result.push(*dep_on);
        }

        result
    }

    fn simplify_impl(&mut self, target : TargetIndex,
                     seen : &mut Vec<bool>) -> Vec<bool> {
        let num_targets = self.targets.len();

        seen[target.0] = true;

        let mut prune = Vec::<bool>::with_capacity(num_targets);
        prune.resize(num_targets, false);

        // fighting the borrow checker
        let down = self.targets[target.0].down.clone();

        let mut below = Vec::<bool>::with_capacity(num_targets);
        below.resize(num_targets, false);
                
        for dependent in down.iter() {
            if !seen[dependent.0] {
                let this_below = self.simplify_impl(*dependent, seen);

                for i in 0..self.targets.len() {
                    below[i] = below[i] || this_below[i];
                }
            }
        }

        self.targets[target.0].down.retain(|&d| !below[d.0]);

        for dependent in down.iter() {
            below[dependent.0] = true;
        }

        below
    }

    pub fn simplify(&mut self) {
        if self.simple {
            return;
        }
        
        let mut seen = Vec::<bool>::with_capacity(self.targets.len());
        seen.resize(self.targets.len(), false);

        let roots = self.roots.clone();
        
        for root in roots.iter() {
            let _below = self.simplify_impl(*root, &mut seen);
        }

        self.simple = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn two_target() {
        let mut deptree = Deptree::<()>::new();
        let a = deptree.add_target("a");
        let b = deptree.add_target("b");
        deptree.depend(b, a);

        assert!(!deptree.done());
        assert_eq!(deptree.name(a), "a");
        assert_eq!(deptree.name(b), "b");

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "a");
        assert!(!deptree.done());

        deptree.start(a).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.finish(a).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "b");
        assert!(!deptree.done());

        deptree.start(b).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.finish(b).unwrap();
        assert!(deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
    }

    #[test]
    fn two_target_fail() {
        let mut deptree = Deptree::<()>::new();
        let a = deptree.add_target("a");
        let b = deptree.add_target("b");
        deptree.depend(b, a);
        assert!(!deptree.done());

        assert_eq!(deptree.name(a), "a");
        assert_eq!(deptree.name(b), "b");

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "a");
        assert!(!deptree.done());

        deptree.start(a).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.fail(a).unwrap();
        assert!(deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
    }

    #[test]
    fn dup_simplify() {
        let mut deptree = Deptree::<()>::new();

        let a = deptree.add_target("a");
        let b = deptree.add_target("b");
        let c = deptree.add_target("c");

        deptree.depend(b, a);
        deptree.depend(c, b);
        deptree.depend(c, a);

        let a_dep_by = deptree.depended_by(a);
        assert_eq!(a_dep_by.len(), 2);
        assert!(!a_dep_by.iter().find(|&i| *i == b).is_none());
        assert!(!a_dep_by.iter().find(|&i| *i == c).is_none());

        deptree.simplify();

        let a_dep_by = deptree.depended_by(a);
        assert_eq!(a_dep_by.len(), 1);
        assert_eq!(a_dep_by[0], b);
        
        let b_dep_by = deptree.depended_by(b);
        assert_eq!(b_dep_by.len(), 1);
        assert_eq!(b_dep_by[0], c);
        
        let c_dep_by = deptree.depended_by(c);
        assert_eq!(c_dep_by.len(), 0);
    }
    
    #[test]
    fn dup_simplify_run() {
        let mut deptree = Deptree::<()>::new();

        let a = deptree.add_target("a");
        let b = deptree.add_target("b");
        let c = deptree.add_target("c");

        deptree.depend(b, a);
        deptree.depend(c, b);
        deptree.depend(c, a);

        assert!(!deptree.done());
        assert_eq!(deptree.name(a), "a");
        assert_eq!(deptree.name(b), "b");
        assert_eq!(deptree.name(c), "c");

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "a");
        assert!(!deptree.done());
        
        deptree.start(a).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.finish(a).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "b");
        assert!(!deptree.done());

        deptree.start(b).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.finish(b).unwrap();
        assert!(!deptree.done());
        
        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "c");
        assert!(!deptree.done());

        deptree.start(c).unwrap();
        assert!(!deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
        assert!(!deptree.done());

        deptree.finish(c).unwrap();
        assert!(deptree.done());

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
    }
}
