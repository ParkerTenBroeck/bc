use std::{collections::{HashMap, HashSet}, num::NonZeroUsize};

use types::{Layout, Type};

use super::parser::ast::Type as UnresolvedType;

use crate::parser::ast::{Expression, FunctionDef, FunctionHeader, Module, Path, Statement};

pub mod scope;
pub mod types;
pub mod constant_eval;

#[derive(Debug)]
pub struct StructMember {
    pub offset: usize,
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Struct {
    layout: Option<Layout>,
    pub members: Vec<StructMember>,
}

#[derive(Debug)]
pub struct EnumVarient {
    pub value: usize,
    pub name: String,
}

#[derive(Debug)]
pub struct Enum {
    layout: Option<Layout>,
    pub members: Vec<EnumVarient>,
}

#[derive(Debug)]
pub struct UnionMember {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Union {
    layout: Option<Layout>,
    pub members: Vec<UnionMember>,
}

#[derive(Debug)]
pub enum UserType {
    Struct(Struct),
    Union(Union),
    Enum(Enum),
    _Processing,
}

pub enum Resolvable<R, U>{
    Resolved(R),
    Unresolved(U),
}

pub enum Global {
    Constant(Resolvable<(Type, ConstantId), (UnresolvedType, UnresolvedConstantId)>),
    Static(Option<Resolvable<ConstantId, UnresolvedConstantId>>),
    Function(FunctionId),
    
    Resolving,
}

#[derive(Default)]
pub struct Context{
    type_map: TypeMap,
    globals: HashMap<Path, Global>,
    constants: Vec<Constant>,
    unresolved_constants: Vec<Constant>,
    functions: Vec<(Resolvable<FunctionSig, UnresolvedFunctionSig>, FunctionKind)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstantId(usize);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnresolvedConstantId(usize);


#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(usize);

pub enum ConstantKind{
    Constant,
    UnNamedArraySize,
    StaticInitialization,
}

pub enum ConstantValue{
    Resolved(constant_eval::Value),
    Unresolved(Expression),
}

pub struct Constant{
    kind: ConstantKind,
    value: ConstantValue,
}

#[derive(Default)]
pub struct Program {
    pub context: Context,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct FunctionSig {
    name: Option<Path>,
    ret_ty: Type,
    params: Vec<(Type, String)>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct UnresolvedFunctionSig {
    name: Option<Path>,
    ret_ty: UnresolvedType,
    params: Vec<(UnresolvedType, String)>,
}

pub enum FunctionKind {
    Declaration(String),
    Definition {
        external: Option<String>,
        code: Vec<Statement>,
    },
}

#[derive(Default)]
pub struct TypeMap {
    pub types: HashMap<Path, UserType>,
}

impl Context{
    pub fn get_type(&mut self, path: &Path) -> (Layout, &UserType) {
        (self.layout(path), self.type_map.types.get(path).unwrap())
    }

    pub fn layout(&mut self, path: &Path) -> Layout {
        let ty = match self.type_map.types.get_mut(path) {
            Some(some) => some,
            None => panic!("type {path:?} not defined"),
        };
        match ty {
            UserType::Struct(struc) => {
                if let Some(layout) = struc.layout {
                    layout
                } else {
                    let mut pty = UserType::_Processing;
                    std::mem::swap(&mut pty, ty);
                    let mut struc = match pty {
                        UserType::Struct(struc) => struc,
                        _ => panic!(),
                    };
                    // let mut layout = Layout::ZERO_SIZE;
                    let mut size = 0;
                    let mut align = NonZeroUsize::MIN;
                    let mut sized = true;

                    for member in &mut struc.members {
                        if !sized {
                            panic!("can only have 1 trailing unsized type");
                        }
                        let ty_layout = member.ty.layout(self);
                        // align the alignment
                        size += ty_layout.align().get() - 1;
                        size &= !(ty_layout.align().get() - 1);

                        member.offset = size;
                        size += ty_layout.size_bytes();
                        align = align.max(ty_layout.align());
                        sized &= ty_layout.is_sized();
                    }

                    let layout = Layout::new_nonzero(size, align).unwrap();

                    struc.layout = Some(layout);
                    *self.type_map.types.get_mut(path).unwrap() = UserType::Struct(struc);

                    layout
                }
            }
            UserType::Union(unio) => {
                if let Some(layout) = unio.layout {
                    layout
                } else {
                    let mut pty = UserType::_Processing;
                    std::mem::swap(&mut pty, ty);
                    let mut unio = match pty {
                        UserType::Union(unio) => unio,
                        _ => panic!(),
                    };
                    let mut layout = Layout::ZERO_SIZE;

                    for member in &mut unio.members {
                        layout = layout.max(member.ty.layout(self))
                    }

                    unio.layout = Some(layout);
                    *self.type_map.types.get_mut(path).unwrap() = UserType::Union(unio);

                    layout
                }
            }
            UserType::Enum(enu) => {
                if let Some(layout) = enu.layout {
                    layout
                } else {
                    let mut pty = UserType::_Processing;
                    std::mem::swap(&mut pty, ty);
                    let mut enu = match pty {
                        UserType::Enum(enu) => enu,
                        _ => panic!(),
                    };

                    #[allow(clippy::match_overlapping_arm)]
                    let layout = match enu.members.len() {
                        0..=1 => Layout::ZERO_SIZE,
                        0..=0xFF => Layout::new(1, 1).unwrap(),
                        0..=0xFFFF => Layout::new(2, 2).unwrap(),
                        0..=0xFFFFFFFF => Layout::new(4, 4).unwrap(),
                        0..=0xFFFFFFFFFFFFFFFF => Layout::new(8, 8).unwrap(),
                        _ => panic!("Enum {path:?} has too many varients!"),
                    };

                    enu.layout = Some(layout);
                    *self.type_map.types.get_mut(path).unwrap() = UserType::Enum(enu);

                    layout
                }
            }

            UserType::_Processing => {
                panic!("Recursive Type!")
            }
        }
    }

    fn resolve_constant(&mut self, id: ConstantId){

    }
    
    fn add_global(&mut self, path: Path, glob: Global) {
 
        // let mut item = self.constant_sups.remove(&path).unwrap_or_default();
        // item.retain(|v|{
        //     let item = self.constant_deps.get_mut(v);
        //     if let Some(map) = item{
        //         map.remove(&path);
        //         map.is_empty()
        //     }else{
        //         true
        //     }
        // });
        // assert!(self.globals.insert(path, glob).is_none());
    }
}


impl Program{
    fn add_function_head(&mut self, mod_path: &Path, func: FunctionHeader){
        // let mut path = mod_path.clone();
        // path.push(&func.name);

        // // self.context.add_glob_func();
        // let sig = FunctionSig {
        //     name: Some(path),
        //     params: func.params,
        //     ret_ty: func.ret.unwrap_or(Type::Void),
        // };

        // self.functions.push(
        //     (sig, FunctionKind::Declaration(func.kind.unwrap_or_default())),
        // );
    }
    
    fn add_function_def(&mut self, mod_path: &Path, func: FunctionDef){
        // let mut path = mod_path.clone();
        // path.push(&func.name);

        // let id = FunctionId(self.functions.len());
        // self.context.add_global(path.clone(), Global::Function(id));

        // self.functions.push(
        //     (
        //         FunctionSig {
        //             name: Some(path),
        //             params: func.params,
        //             ret_ty: func.ret.unwrap_or(Type::Void),
        //         },
        //         FunctionKind::Definition {
        //             external: func.kind,
        //             code: func.body,
        //         },
        //     ),
        // );
    }

    pub fn load_module(&mut self, mod_path: Path, module: Module){
        for _use_smt in module.use_statements{

        }
        // module.items.
        // for item in module.items {
        //     match item {
        //         crate::parser::ast::TopLevelDef::StructDef(struc) => {
        //             let mut path = mod_path.clone();
        //             path.push(&struc.name);
    
        //             let def = Struct {
        //                 layout: None,
        //                 members: struc
        //                     .values
        //                     .into_iter()
        //                     .map(|(ty, name)| StructMember {
        //                         offset: 0,
        //                         name,
        //                         ty,
        //                     })
        //                     .collect(),
        //             };
    
        //             self.context.type_map.types.insert(path, UserType::Struct(def));
        //         }
        //         crate::parser::ast::TopLevelDef::EnumDef(enu) => {
        //             let mut path = mod_path.clone();
        //             path.push(&enu.name);
    
        //             let def = Enum {
        //                 layout: None,
        //                 members: enu
        //                     .values
        //                     .into_iter()
        //                     .enumerate()
        //                     .map(|(value, name)| EnumVarient { value, name })
        //                     .collect(),
        //             };
    
        //             self.context.type_map.types.insert(path, UserType::Enum(def));
        //         }
        //         crate::parser::ast::TopLevelDef::UnionDef(unio) => {
        //             let mut path = mod_path.clone();
        //             path.push(&unio.name);
    
        //             let def = Union {
        //                 layout: None,
        //                 members: unio
        //                     .values
        //                     .into_iter()
        //                     .map(|(ty, name)| UnionMember { name, ty })
        //                     .collect(),
        //             };
    
        //             self.context.type_map.types.insert(path, UserType::Union(def));
        //         }
    
        //         crate::parser::ast::TopLevelDef::GlobalDef(glob) => {
        //             // let mut path = mod_path.clone();
        //             // path.push(&glob.name);
        //             // self.globals.insert(
        //             //     path,
        //             //     Global {
        //             //         constant: matches!(glob.kind, crate::parser::ast::GlobalKind::Const),
        //             //         ty: glob.ty,
        //             //         expr: glob.value,
        //             //     },
        //             // );
        //         }
    
        //         crate::parser::ast::TopLevelDef::FunctionDef(func) => {
        //             self.add_function_def(&mod_path, func);
        //         }
        //         crate::parser::ast::TopLevelDef::FunctionHeader(func) => {
        //             self.add_function_head(&mod_path, func);
        //         }
    
        //         crate::parser::ast::TopLevelDef::UseStatement() => todo!(),
        //     }
        // }
    }

    pub fn check_invalid_unsized(&mut self){
        // for (sig, _ ) in &mut self.functions{
        //     assert!(sig.ret_ty.layout(&mut self.context).is_sized(), "Function return values must be sized");
        //     for param in sig.params.iter().map(|v|&v.0){
        //         assert!(param.layout(&mut self.context).is_sized(), "Function parameters must be sized");
        //     }
        // }

        // for glob in self.globals.values(){
        //     assert!(glob.ty.layout(&mut self.context).is_sized(), "Global values must be sized");
        // }
    }

    // pub fn generate_global_values(&mut self){
    //     for glob in self.globals.values_mut(){
            
    //     }
    // }
}


#[test]
fn test() {
    let mut program = Program::default();

    let parser = crate::parser::def::ModuleParser::new();
    let str = include_str!("../../test/main.bc");    

    let res = parser.parse(str).unwrap();

    let module = Path::new();

    program.load_module(module, res);
    program.check_invalid_unsized();
    program.generate_global_values();

    println!("{:#?}", program.type_map.get_type(&Path::new_path("Other")));
    println!("{:#?}", program.type_map.get_type(&Path::new_path("Thing")));
}
