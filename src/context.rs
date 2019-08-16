use super::*;

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq, Eq,
)]
pub
struct Context {
    depth: u8,
}

impl Context {
    #[inline]
    pub
    fn new ()
        -> Self
    {
        Self::default()
    }

    pub
    fn deeper (self: &'_ Self)
        -> Self
    {
        Self {
            depth: self.depth.saturating_add(1),
            .. self.clone()
        }
    }

    pub
    fn indent (self: &'_ Self)
        -> impl fmt::Display + '_
    {
        (0 .. self.depth).map(|_| "    ").join("")
    }
}