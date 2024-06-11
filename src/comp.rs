use std::collections::HashMap;

use crate::stage::scope::Scope;

struct Workspace {
    files: HashMap<Scope<'static>, String>,
}
