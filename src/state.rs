use domain_types::User;

struct State {
  users: Vec<User>
}

impl State {
  pub fn new() -> Self {
    State { users: vec![] }
  }
}
