
enum TargetState {
    Unstarted,
    Running,
    Failed,
    Finished
}

pub struct TargetIndex(usize);

type Links = Vec<TargetIndex>;

struct TargetImpl<Attribs> {
    name : String,
    state : TargetState,
    dependencies : Links,
    depended_by : Links,
    attribs : Option<Attribs>
}

impl<Attribs> TargetImpl<Attribs> {
    fn new(name: String, attribs : Option<Attribs>) -> Self {
        Self { name : name,
               state : TargetState::Unstarted,
               dependencies : Links::new(),
               depended_by : Links::new(),
               attribs : attribs
        }
    }
}

type TargetImpls<Attribs> = Vec<TargetImpl<Attribs>>;

pub struct Deptree<Attribs = ()> {
    targets : TargetImpls<Attribs>
}

pub struct Target<'a, Attribs> {
    index : TargetIndex,
    tree : & 'a Deptree<Attribs>
}

impl<Attribs> Deptree<Attribs> {
    pub fn new() -> Deptree { Deptree { targets : TargetImpls::new() } }

    pub fn add_target_attribs<'a>(&'a mut self, name: &str, attribs : Option<Attribs>)
                              -> Target<'a, Attribs> {
        self.targets.push(TargetImpl::new(name.to_string(), attribs));
        Target { index : TargetIndex(self.targets.len() - 1), tree : self}
    }
    
    pub fn add_target(&mut self, name: &str) -> Target<Attribs> {
        self.add_target_attribs(name, None)
    }
}

/*impl<Attribs> Target<Attribs> {
    fn depend(other : Target<Attribs>) {
        
    }
}*/

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn it_works() {
        let mut deptree = Deptree::<()>::new();
        let target = deptree.add_target("blah");
    }
}
