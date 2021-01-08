use crate::{CompatibleStage, Expr, Return, Scope, ToType, Var};
use do_notation::Lift;
use std::marker::PhantomData;

/// Monadic version of [`Scope`].
pub struct MScope<S, R, A> {
  scope: Box<dyn FnOnce(&mut Scope<S, R>) -> A>,
  _phantom: PhantomData<(S, R, A)>,
}

impl<S, R, A> Lift<A> for MScope<S, R, A>
where
  A: 'static,
{
  fn lift(value: A) -> Self {
    Self {
      scope: Box::new(move |_| value),
      _phantom: PhantomData,
    }
  }
}

impl<S, R, A> MScope<S, R, A>
where
  A: 'static,
  S: 'static,
  R: 'static,
{
  pub fn and_then<B>(self, f: impl FnOnce(A) -> MScope<S, R, B> + 'static) -> MScope<S, R, B> {
    let scope: Box<dyn FnOnce(&mut Scope<S, R>) -> B> = Box::new(move |s| {
      let a = (self.scope)(s);
      (f(a).scope)(s)
    });

    MScope {
      scope,
      _phantom: PhantomData,
    }
  }
}

pub fn var<S, R, Q, T>(init_value: impl Into<Expr<Q, T>> + 'static) -> MScope<S, R, Var<S, T>>
where
  S: CompatibleStage<Q>,
  T: ToType,
  Return<S>: From<R>,
{
  let scope: Box<dyn FnOnce(&mut Scope<S, R>) -> Var<S, T>> = Box::new(|s| s.var(init_value));

  MScope {
    scope,
    _phantom: PhantomData,
  }
}

pub fn leave<S, R>(ret: impl Into<R> + 'static) -> MScope<S, R, ()>
where
  Return<S>: From<R>,
{
  let scope: Box<dyn FnOnce(&mut Scope<S, R>) -> ()> = Box::new(|s| s.leave(ret));

  MScope {
    scope,
    _phantom: PhantomData,
  }
}

pub fn abort<S, R>() -> MScope<S, R, ()>
where
  S: 'static,
  R: 'static,
  Return<S>: From<R>,
{
  let scope: Box<dyn FnOnce(&mut Scope<S, R>) -> ()> = Box::new(Scope::abort);

  MScope {
    scope,
    _phantom: PhantomData,
  }
}

#[cfg(test)]
mod tests {
  use crate::L;

  use super::*;
  use do_notation::m;

  #[test]
  fn mscope_var() {
    let _scope: MScope<L, (), Expr<L, i32>> = m! {
      x <- var(1);
      y <- var(2);
      return x.to_expr() + y.to_expr();
    };
  }
}
