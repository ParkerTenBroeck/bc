use std::{
    any::TypeId,
    marker::PhantomData,
    mem::{size_of, MaybeUninit},
};

use lalrpop_util::lalrpop_mod;

macro_rules! operator {
    (reverseME $output:ident => $par:ident: $par_ty:ty, $(
        $par_r:ident: $par_ty_r:ty,
    ),* $(,)?) => {
        operator!(reverseME $output => $($par_r: $par_ty_r,)*);
        let $par: $par_ty = unsafe{$output.pop()};
    };
    (reverseME $output:ident => ) => {
    };

    (TUP($ident:ident) $ret:ty $(,)?) => {
        ($ident,)
    };
    (TUP($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $ident
    };

    (0 $output:ident($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $output.push($ident.0);
        operator!(1 $output($ident) $($ret_r,)*);
    };
    (1 $output:ident($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $output.push($ident.1);
        operator!(2 $output($ident) $($ret_r,)*);
    };
    (2 $output:ident($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $output.push($ident.2);
        operator!(3 $output($ident) $($ret_r,)*);
    };
    (3 $output:ident($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $output.push($ident.3);
        operator!(4 $output($ident) $($ret_r,)*);
    };
    (4 $output:ident($ident:ident) $ret:ty,$($ret_r:ty,)*) => {
        $output.push($ident.4);
        operator!(5 $output($ident) $($ret_r,)*);
    };
    ($lit:literal $output:ident($ident:ident)) => {
    };

    (   $context:ident: $context_ty:ty;

        $(#[$meta:meta])* $vis:vis enum $name:ident {
        $(
            $op:ident(
                $(
                    $par:ident: $par_ty:ty
                ),* $(,)?
            ) -> $($ret:ty),* $(,)? $expr:block
        )*
    }
    $(#[$meta_o:meta])* $vis_o:vis enum $name_o:ident {
        $(
            $ty_e_name:ident($ty:ty)
        ),* $(,)?
    }
    ) => {
        $(#[$meta])*
        $vis enum $name{
            $(
                $op,
            )*
        }


        $(#[$meta_o])* $vis_o enum $name_o {
            $(
                $ty_e_name($ty)
            ),*
        }

        unsafe impl Operator for $name{
            type Context = $context_ty;
            type AnyValue = $name_o;

            unsafe fn run(&self, $context: &mut Self::Context, output: &mut HorribleVec) {
                match self{
                    $(
                        Self::$op => {
                            operator!(reverseME output => $($par: $par_ty,)*);

                            #[allow(unused_parens)]
                            let tup: ($($ret),*);
                            tup = $expr;
                            let tup: ($($ret,)*) = operator!(TUP(tup)$($ret,)*);
                            operator!(0 output(tup) $($ret,)*);
                        }
                    )*
                };
            }

            fn output(&self) -> &'static [TypeId] {
                match self{
                    $(
                        Self::$op => {
                            const V: &[TypeId] = &[$(TypeId::of::<$ret>(),)*];
                            V
                        }
                    )*
                }
            }

            fn input(&self) -> &'static [TypeId] {
                match self{
                    $(
                        Self::$op => {
                            const V: &[TypeId] = &[$(TypeId::of::<$par_ty>(),)*];
                            V
                        }
                    )*
                }
            }

            unsafe fn get_value(ty: TypeId, output: &mut HorribleVec) -> $name_o{
                $(
                    #[allow(non_upper_case_globals)]
                    const $ty_e_name: TypeId = TypeId::of::<$ty>();
                )*
                match ty{
                    $(
                        val if val == $ty_e_name => $name_o::$ty_e_name(output.pop::<$ty>()),
                    )*
                    _ => todo!()
                }
            }
        }
    };
}

pub unsafe trait Operator {
    type AnyValue;
    type Context;

    fn output(&self) -> &[TypeId];
    fn input(&self) -> &[TypeId];
    unsafe fn run(&self, context: &mut Self::Context, output: &mut HorribleVec);
    unsafe fn get_value(ty: TypeId, output: &mut HorribleVec) -> Self::AnyValue;
}

operator! {
    _context: ();

    #[derive(Debug)]
    pub enum BasicOperator {
        Times(left: f64, right: f64) -> f64{
            left * right
        }
        Div(left: f64, right: f64) -> f64{
            left / right
        }
        Add(left: f64, right: f64) -> f64{
            left + right
        }
        Minus(left: f64, right: f64) -> f64{
            left - right
        }

        Eq(left: f64, right: f64) -> bool{
            left == right
        }
        Gt(left: f64, right: f64) -> bool{
            left > right
        }
        Lt(left: f64, right: f64) -> bool{
            left < right
        }

        Or(left: bool, right: bool) -> bool{
            left || right
        }
        And(left: bool, right: bool) -> bool{
            left && right
        }
    }
    #[derive(Debug, Clone, Copy)]
    pub enum Type{
        Number(f64),
        Boolean(bool),
    }
}

#[derive(Default, Debug)]
pub struct HorribleVec {
    inner: Vec<MaybeUninit<u8>>,
}

impl HorribleVec {
    pub const fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn push<T>(&mut self, value: T) {
        self.inner.reserve(size_of::<T>());
        unsafe {
            self.inner
                .as_mut_ptr()
                .add(self.inner.len())
                .cast::<T>()
                .write_unaligned(value);
            self.inner.set_len(self.inner.len() + size_of::<T>());
        }
    }

    pub fn push_front<T>(&mut self, value: T) {
        self.inner.reserve(size_of::<T>());
        unsafe {
            std::ptr::copy(
                self.inner.as_ptr(),
                self.inner.as_mut_ptr().add(size_of::<T>()),
                self.inner.len(),
            );

            self.inner.as_mut_ptr().cast::<T>().write_unaligned(value);

            self.inner.set_len(self.inner.len() + size_of::<T>());
        }
    }

    pub unsafe fn pop<T>(&mut self) -> T {
        let data = self.inner[self.inner.len() - size_of::<T>()..self.inner.len()].as_ptr();
        let value = data.cast::<T>().read_unaligned();
        for _ in 0..size_of::<T>() {
            self.inner.pop().unwrap();
        }
        value
    }

    pub unsafe fn push_n_from(&mut self, values: &mut HorribleVec, size: usize) {
        self.inner.reserve(size);
        assert!(values.inner.len() >= size);
        unsafe {
            let src = values.inner.as_mut_ptr().add(values.inner.len() - size);
            let dst = self.inner.as_mut_ptr().add(self.inner.len());
            std::ptr::copy_nonoverlapping(src, dst, size);

            values.inner.set_len(values.inner.len() - size);
            self.inner.set_len(self.inner.len() + size);
        }
    }
}

#[test]
fn bruh() {
    let mut thing = HorribleVec::new();

    thing.push_front(12i32);
    thing.push_front("Hiiii!");
    thing.push_front("asldkasd".to_owned());

    unsafe {
        println!("{}", thing.pop::<i32>());
        println!("{}", thing.pop::<&str>());
        println!("{}", thing.pop::<String>());
    }
}

#[derive(Debug)]
pub enum MachineOperation<O> {
    Lit(TypeId, usize),
    Op(O),
}

#[derive(Default, Debug)]
pub struct Run<O: Operator> {
    pub values: HorribleVec,
    stack: HorribleVec,
    pub operator: Vec<MachineOperation<O>>,
}

impl<O: Operator> Run<O> {
    pub fn new() -> Self {
        Self {
            operator: Default::default(),
            stack: Default::default(),
            values: Default::default(),
        }
    }

    pub fn type_check(&self) -> Result<(), ()> {
        let mut stack = Vec::new();

        for op in self.operator.iter().rev() {
            if !stack.is_empty() {
                let types = match op {
                    MachineOperation::Lit(t, _) => &[*t],
                    MachineOperation::Op(o) => o.output(),
                };
                for ty in types {
                    if Some(*ty) != stack.pop() {
                        return Err(());
                    }
                }
            }
            match op {
                MachineOperation::Lit(_, _) => {}
                MachineOperation::Op(op) => {
                    stack.extend_from_slice(op.input());
                }
            }
        }
        if stack.is_empty() {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn add_operator(&mut self, op: O) {
        self.operator.push(MachineOperation::Op(op));
    }

    pub fn push_val<T: 'static>(&mut self, value: T) {
        self.operator
            .push(MachineOperation::Lit(TypeId::of::<T>(), size_of::<T>()));
        self.values.push_front(value);
    }

    pub fn run(&mut self, context: &mut O::Context) {
        for op in self.operator.iter() {
            match op {
                MachineOperation::Lit(_, size) => unsafe {
                    self.stack.push_n_from(&mut self.values, *size);
                },
                MachineOperation::Op(op) => unsafe {
                    op.run(context, &mut self.stack);
                },
            }
        }
    }

    pub fn results(&mut self) -> impl Iterator<Item = O::AnyValue> + '_ {
        struct ResultIter<'a, O: Operator> {
            results: &'a mut HorribleVec,
            types: std::slice::Iter<'a, TypeId>,
            _p: PhantomData<O>,
        }

        impl<'a, O: Operator> Iterator for ResultIter<'a, O> {
            type Item = O::AnyValue;

            fn next(&mut self) -> Option<Self::Item> {
                Some(unsafe { O::get_value(*self.types.next()?, &mut self.results) })
            }
        }

        if let Some(last) = self.operator.last() {
            ResultIter {
                results: &mut self.stack,
                types: match last {
                    MachineOperation::Lit(ty, _) => std::slice::from_ref(ty).into_iter(),
                    MachineOperation::Op(op) => op.output().iter(),
                },
                _p: PhantomData::<O>,
            }
        } else {
            ResultIter {
                results: &mut self.stack,
                types: [].iter(),
                _p: PhantomData,
            }
        }
    }
}

lalrpop_mod!(calc2);

#[test]
fn test4() {
    let mut program = Run::new();

    let input = "(2+3*4) + 12";
    calc2::FinishedParser::new()
        .parse(&mut program, input)
        .unwrap();
    println!("{:#?}", program.type_check());
    program.run(&mut ());
    println!("{:#?}", program.results().collect::<Vec::<_>>());
}

#[test]
fn testtest() {
    #[derive(Debug)]
    enum Instruction {
        Lit,
        Op,
    }

    impl Instruction {
        fn children(&self) -> usize {
            match self {
                Instruction::Lit => 0,
                Instruction::Op => 2,
            }
        }
    }

    let instructions = vec![
        Instruction::Op,
        Instruction::Op,
        Instruction::Lit,
        Instruction::Op,
        Instruction::Lit,
        Instruction::Lit,
        Instruction::Op,
        Instruction::Op,
        Instruction::Lit,
        Instruction::Lit,
        Instruction::Lit,
    ];

    let mut count: Vec<usize> = Vec::new();
    for instruction in instructions {
        let children: usize = instruction.children();

        while let Some(v) = count.last_mut() {
            if *v == 0 {
                print!(")");
                count.pop();
            } else {
                *v -= 1;
                break;
            }
        }
        if children != 0 {
            print!("(");
            count.push(children);
        }
        print!("{instruction:?} ");
    }
    while let Some(v) = count.last_mut() {
        if *v == 0 {
            print!(")");
            count.pop();
        } else {
            panic!();
        }
    }
}
