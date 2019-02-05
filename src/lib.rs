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
    AlreadyFailed(String)
}

type DeptreeResult<T> = Result<T, DeptreeError>;

enum TargetState {
    Unstarted,
    Started,
    Failed,
    Finished
}

#[derive(Hash, Clone, Copy, Eq, PartialEq)]
pub struct TargetIndex(usize);

type Indices = HashSet<TargetIndex>;
type IndexList = Vec<TargetIndex>;

struct TargetData<Attribs> {
    name : String,
    state : TargetState,
    dependencies : Indices,
    depended_by : Indices,
    attribs : Option<Attribs>
}

impl<Attribs> TargetData<Attribs> {
    fn new(name: String, attribs : Option<Attribs>) -> Self {
        Self { name : name,
               state : TargetState::Unstarted,
               dependencies : Indices::new(),
               depended_by : Indices::new(),
               attribs : attribs
        }
    }
}

type Targets<Attribs> = Vec<TargetData<Attribs>>;

pub struct Deptree<Attribs = ()> {
    targets : Targets<Attribs>,
    roots : Indices
}

impl<Attribs> Deptree<Attribs> {
    pub fn new() -> Deptree {
        Deptree {
            targets : Targets::new(),
            roots : Indices::new()
        }
    }

    pub fn add_target_attribs<'a>(&'a mut self, name: &str, attribs : Option<Attribs>)
                              -> TargetIndex {
        self.targets.push(TargetData::new(name.to_string(), attribs));
        let index = TargetIndex(self.targets.len() - 1);
        self.roots.insert(index);
        index
    }
    
    pub fn add_target(&mut self, name: &str) -> TargetIndex {
        self.add_target_attribs(name, None)
    }

    pub fn depend(&mut self, one : TargetIndex, two : TargetIndex) {
        self.targets[one.0].dependencies.insert(two);
        self.targets[two.0].depended_by.insert(one);
        self.roots.remove(&one);
    }

    pub fn name(&self, target : TargetIndex) -> &String {
        &self.targets[target.0].name
    }

    pub fn attribs(&self, target : TargetIndex) -> &Option<Attribs> {
        &self.targets[target.0].attribs
    }

    pub fn ready(&self) -> IndexList {
        let mut result = IndexList::new();
        for i in self.roots.iter() {
            result.push(*i);
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
                self.roots.remove(&target);
                Ok(())
            }
        }
    }

    pub fn finish(&mut self, target : TargetIndex) -> DeptreeResult<()> {
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
                for dependent in data.depended_by.iter() {
                    self.roots.insert(*dependent);
                }
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        let mut deptree = Deptree::<()>::new();
        let a = deptree.add_target("a");
        let b = deptree.add_target("b");
        deptree.depend(b, a);

        assert_eq!(deptree.name(a), "a");
        assert_eq!(deptree.name(b), "b");

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "a");

        deptree.start(a).unwrap();

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);

        deptree.finish(a).unwrap();

        let ready = deptree.ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(deptree.name(ready[0]), "b");

        deptree.start(b).unwrap();

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);

        deptree.finish(b).unwrap();

        let ready = deptree.ready();
        assert_eq!(ready.len(), 0);
    }
}
