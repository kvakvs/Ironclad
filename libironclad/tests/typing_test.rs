extern crate function_name;
extern crate libironclad_erlang;

mod test_util;

use ::function_name::named;
use libironclad_erlang::error::ic_error::IcResult;
use libironclad_erlang::project::module::scope::scope_impl::{Scope, ScopeImpl};
use libironclad_erlang::typing::check::TypeCheck;
use libironclad_erlang::typing::erl_type::ErlType;
use std::ops::Deref;

#[named]
#[test]
fn typing_synth() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.Synth");

  {
    let scope1 = ScopeImpl::new_root_scope(function_name!().to_string());
    let expr1 = test_util::parse_expr(function_name!(), "[3.14159265358979 , 2,atom]");
    let synth_t1 = expr1.synthesize(module, &scope1)?;
    println!("Synth list1: {}", &synth_t1);

    if let ErlType::StronglyTypedList { elements, tail } = synth_t1.deref() {
      assert!(elements[0].is_float());
      assert!(elements[1].is_integer());
      assert!(elements[2].is_atom());

      let tail2 = tail.clone().unwrap_or_else(|| ErlType::nil());
      assert!(tail2.is_nil());
      assert!(tail2.is_list());
    } else {
      panic!("Expected: StronglyTypedList, got {}", synth_t1)
    }
  }

  {
    let scope2 = ScopeImpl::new_root_scope(function_name!().to_string());
    let expr2 = test_util::parse_expr(function_name!(), "{tuple_tag, 1.2, 3, \"hello\"}");
    let synth_t2 = expr2.synthesize(module, &scope2)?;
    println!("Synth tup1: {}", &synth_t2);

    if let ErlType::Tuple { elements } = synth_t2.deref() {
      assert!(
        elements[0].is_lit_atom("tuple_tag"),
        "t[0] - expected 'tuple_tag', got {}",
        elements[0]
      );
      assert!(elements[1].is_float(), "t[1] - expected float, got {}", elements[1]);
      assert!(elements[2].is_integer(), "t[2] - expected integer, got {}", elements[2]);
      assert!(elements[3].is_list(), "t[3] - expected string, got {}", elements[3]);
    } else {
      panic!("Expected: Tuple, got {}", synth_t2)
    }
  }

  Ok(())
}

#[named]
#[test]
fn typing_expr_check_1() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.ExprCheck.Atom");
  let scope = ScopeImpl::new_root_scope(function_name!().to_string());
  let expr = test_util::parse_expr(function_name!(), "hello");
  assert!(
    TypeCheck::check(module, &scope, &expr, &ErlType::Atom)?,
    "Parsed atom 'hello' must be subtype of atom()"
  );
  Ok(())
}

#[named]
#[test]
/// Create a fun with 0 args, which returns an integer(). See if its compatible with an integer().
fn typing_expr_check_noarg() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.ExprCheck.IntegerFun");

  let scope = ScopeImpl::new_root_scope(function_name!().to_string());
  let nodes = test_util::parse_module_unwrap(function_name!(), "my_int_fun1() -> 10 + 20.");
  assert!(nodes[0].is_fn_def(), "Expected FnDef() received {:?}", nodes);

  let match_ty = &ErlType::new_fn_type_of_any_args(0, &ErlType::integer());
  assert!(
    TypeCheck::check(module, &scope, &nodes[0], match_ty)?,
    "my_int_fun1()'s return type must be compatible with integer()"
  );
  Ok(())
}

#[named]
#[test]
/// Create a fun with argument, which returns an integer(). See if its compatible with an integer().
fn typing_check_int_arg_fn() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.ExprCheck.IntegerFunWithArg");
  let scope = ScopeImpl::new_root_scope(function_name!().to_string());
  let nodes = test_util::parse_module_unwrap(function_name!(), "my_int_fun2(A) -> 10 + A.");

  assert!(nodes[0].is_fn_def(), "Expected FnDef() received {:?}", nodes);
  // println!("Synth my_int_fun2: {}", int_fn2.core_ast.synthesize(&env)?);

  let match_ty = &ErlType::new_fn_type_of_any_args(1, &ErlType::integer());
  assert!(
    TypeCheck::check(module, &scope, &nodes[0], match_ty)?,
    "my_int_fun2()'s result type must be compatible with integer()"
  );
  Ok(())
}

#[named]
#[test]
/// Create a fun which returns a tuple of `{any(), integer()}` and see if it checks against a tuple
fn typing_expr_check_tuple1() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.ExprCheck.TupleFun");
  let scope = ScopeImpl::new_root_scope(function_name!().to_string());
  let nodes = test_util::parse_module_unwrap(function_name!(), "mytuple_fun(A) -> {A, 123}.");

  assert!(nodes[0].is_fn_def(), "Expected FnDef() received {:?}", nodes[0]);
  // println!("Synth mytuple_fun: {}", tuple_fn.core_ast.synthesize(&env)?);

  let expected_type = ErlType::new_tuple(&vec![ErlType::any(), ErlType::integer()]);
  let match_ty = &ErlType::new_fn_type_of_any_args(1, &expected_type);
  assert!(
    TypeCheck::check(module, &scope, &nodes[0], match_ty)?,
    "Parsed mytuple_fun(A) result type must match {{any(), integer()}}"
  );
  Ok(())
}

#[named]
#[test]
fn typing_subtyping_bool() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.Subtyping.Bool");

  let test1_bool = ErlType::Boolean;
  let test1_atom = ErlType::Atom;
  let test1_true = ErlType::new_atom("true");

  assert!(test1_bool.is_subtype_of(&test1_atom));
  assert!(!test1_atom.is_subtype_of(&test1_bool));

  assert!(test1_true.is_subtype_of(&test1_atom));
  assert!(test1_true.is_subtype_of(&test1_bool));
  Ok(())
}

#[named]
#[test]
fn typing_subtyping_number() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.Subtyping.Number");
  let test2_int = ErlType::Integer;
  let test2_flt = ErlType::Float;
  let test2_num = ErlType::Number;

  assert!(test2_int.is_subtype_of(&test2_num)); // int() is subtype of number()
  assert!(!test2_num.is_subtype_of(&test2_int)); // number() is not subtype of int

  assert!(test2_flt.is_subtype_of(&test2_num)); // float() is subtype of number()
  Ok(())
}

#[named]
#[test]
fn typing_subtyping_list() -> IcResult<()> {
  test_util::start(function_name!(), "Typing.Subtyping.List");

  let test3_any = ErlType::AnyList;
  let test3_l_num = ErlType::list_of(ErlType::number());
  let test3_l_flt = ErlType::list_of(ErlType::float());
  let test3_l_int = ErlType::list_of(ErlType::integer());

  assert!(test3_l_num.is_subtype_of(&test3_any)); // list(number()) is subtype of list()
  assert!(!test3_any.is_subtype_of(&test3_l_num)); // list() not subtype of list(number())

  assert!(test3_l_flt.is_subtype_of(&test3_any)); // list(float()) is subtype of list()
  assert!(!test3_any.is_subtype_of(&test3_l_flt)); // list() not subtype of list(float())

  assert!(test3_l_int.is_subtype_of(&test3_any)); // list(integer()) is subtype of list()
  assert!(!test3_any.is_subtype_of(&test3_l_int)); // list() not subtype of list(integer())

  Ok(())
}
