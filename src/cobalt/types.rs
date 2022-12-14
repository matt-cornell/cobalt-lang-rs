use inkwell::types::{BasicType, BasicTypeEnum::{self, *}};
use crate::*;
use Type::{*, Char, Int};
use SizeType::*;
use std::fmt::*;
use std::io::{self, Write, Read, BufRead};
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy)]
pub enum SizeType {
    Static(u64),
    Dynamic,
    Meta
}
impl SizeType {
    pub fn is_static(self) -> bool {if let Static(_) = self {true} else {false}}
    pub fn is_dynamic(self) -> bool {self == Dynamic}
    pub fn is_meta(self) -> bool {self == Meta}
    pub fn as_static(self) -> Option<u64> {if let Static(x) = self {Some(x)} else {None}}
    pub fn map_static<F: FnOnce(u64) -> u64>(self, f: F) -> SizeType {if let Static(x) = self {Static(f(x))} else {self}}
}
#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    IntLiteral, Char,
    Int(u64, bool),
    Float16, Float32, Float64, Float128,
    Pointer(Box<Type>, bool), Reference(Box<Type>, bool), Borrow(Box<Type>),
    Null, Module, TypeData, Array(Box<Type>, Option<u64>),
    Function(Box<Type>, Vec<(Type, bool)>)
}
impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            IntLiteral => write!(f, "<int literal>"),
            Int(size, false) => write!(f, "i{size}"),
            Int(size, true) => write!(f, "u{size}"),
            Char => write!(f, "char"),
            Float16 => write!(f, "f16"),
            Float32 => write!(f, "f32"),
            Float64 => write!(f, "f64"),
            Float128 => write!(f, "f128"),
            Pointer(x, false) => write!(f, "*const {}", *x),
            Pointer(x, true) => write!(f, "*mut {}", *x),
            Reference(x, false) => write!(f, "&const {}", *x),
            Reference(x, true) => write!(f, "&mut {}", *x),
            Borrow(x) => write!(f, "^{}", *x),
            Null => write!(f, "null"),
            Module => write!(f, "module"),
            TypeData => write!(f, "type"),
            Array(x, None) => write!(f, "{}[]", *x),
            Array(x, Some(s)) => write!(f, "{}[{s}]", *x),
            Function(ret, args) => {
                write!(f, "fn (")?;
                let len = args.len();
                for (arg, ty) in args.iter() {
                    write!(f, "{}{}", match ty {
                        true => "const ",
                        false => ""
                    }, arg)?;
                    if len > 1 {write!(f, ", ")?}
                }
                write!(f, "): {}", *ret)
            }
        }
    }
}
impl Type {
    pub fn size(&self) -> SizeType {
        match self {
            IntLiteral => Static(8),
            Int(size, _) => Static((size + 7) / 8),
            Char => Static(4),
            Float16 => Static(2),
            Float32 => Static(4),
            Float64 => Static(8),
            Float128 => Static(16),
            Null => Static(0),
            Array(b, Some(s)) => b.size().map_static(|x| x * s),
            Array(_, None) => Dynamic,
            Function(..) | Module | TypeData => Meta,
            Pointer(..) | Reference(..) => Static(8),
            Borrow(b) => b.size()
        }
    }
    pub fn align(&self) -> u64 {
        match self {
            IntLiteral => 8,
            Int(size, _) => match size {
                0..=8 => 1,
                9..=16 => 2,
                17..=32 => 4,
                33.. => 8
            },
            Char => 4,
            Float16 => 2,
            Float32 => 4,
            Float64 | Float128 => 8,
            Null => 1,
            Array(b, _) => b.align(),
            Function(..) | Module | TypeData => 0,
            Pointer(..) | Reference(..) => 8,
            Borrow(b) => b.align()
        }
    }
    pub fn llvm_type<'ctx>(&self, ctx: &CompCtx<'ctx>) -> Option<BasicTypeEnum<'ctx>> {
        match self {
            IntLiteral => Some(IntType(ctx.context.i64_type())),
            Int(size, _) => Some(IntType(ctx.context.custom_width_int_type(*size as u32))),
            Char => Some(IntType(ctx.context.i32_type())),
            Float16 => Some(FloatType(ctx.context.f16_type())),
            Float32 => Some(FloatType(ctx.context.f32_type())),
            Float64 => Some(FloatType(ctx.context.f64_type())),
            Float128 => Some(FloatType(ctx.context.f128_type())),
            Null | Function(..) | Module | TypeData => None,
            Array(_, Some(_)) => todo!("arrays aren't implemented yet"),
            Array(_, None) => todo!("arrays aren't implemented yet"),
            Pointer(b, _) | Reference(b, _) => Some(PointerType(b.llvm_type(ctx)?.ptr_type(inkwell::AddressSpace::from(0u16)))),
            Borrow(b) => b.llvm_type(ctx)
        }
    }
    pub fn register(&self) -> bool {
        match self {
            IntLiteral | Int(_, _) | Char | Float16 | Float32 | Float64 | Float128 | Null | Function(..) | Pointer(..) | Reference(..) => true,
            Borrow(b) => b.register(),
            _ => false
        }
    }
    pub fn copyable(&self) -> bool {
        match self {
            IntLiteral | Int(_, _) | Char | Float16 | Float32 | Float64 | Float128 | Null | Function(..) | Pointer(..) | Reference(..) | Borrow(_) => true,
            _ => false
        }
    }
    pub fn save<W: Write>(&self, out: &mut W) -> io::Result<()> {
        match self {
            IntLiteral => panic!("There shouldn't be an int literal in a variable!"),
            Int(s, u) => {
                out.write_all(&[1])?;
                let mut v = *s as i64;
                if *u {v = -v;}
                out.write_all(&v.to_be_bytes())
            },
            Char => out.write_all(&[2]),
            Float16 => out.write_all(&[3]),
            Float32 => out.write_all(&[4]),
            Float64 => out.write_all(&[5]),
            Float128 => out.write_all(&[6]),
            Null => out.write_all(&[7]),
            Pointer(b, false) => {
                out.write_all(&[8])?;
                b.save(out)
            },
            Pointer(b, true) => {
                out.write_all(&[9])?;
                b.save(out)
            },
            Reference(b, false) => {
                out.write_all(&[10])?;
                b.save(out)
            },
            Reference(b, true) => {
                out.write_all(&[11])?;
                b.save(out)
            },
            Borrow(b) => {
                out.write_all(&[12])?;
                b.save(out)
            },
            Function(b, p) => {
                out.write_all(&[13])?;
                out.write_all(&(p.len() as u64).to_be_bytes())?; // # of params
                b.save(out)?;
                for (par, c) in p {
                    par.save(out)?;
                    out.write_all(&[if *c {1} else {0}])?; // param is const
                }
                Ok(())
            },
            Module => todo!("Modules can't be stored in variables yet!"),
            TypeData => todo!("Types can't be stored in variables yet!"),
            Array(..) => todo!("Arrays aren't implemented yet!")
        }
    }
    pub fn load<R: Read + BufRead>(buf: &mut R) -> io::Result<Self> {
        let mut c = 0u8;
        buf.read_exact(std::slice::from_mut(&mut c))?;
        Ok(match c {
            1 => {
                let mut bytes = [0; 8];
                buf.read_exact(&mut bytes)?;
                let v = i64::from_be_bytes(bytes);
                Type::Int(v.abs() as u64, v < 0)
            },
            2 => Type::Char,
            3 => Type::Float16,
            4 => Type::Float32,
            5 => Type::Float64,
            6 => Type::Float128,
            7 => Type::Null,
            8 => Type::Pointer(Box::new(Type::load(buf)?), false),
            9 => Type::Pointer(Box::new(Type::load(buf)?), true),
            10 => Type::Reference(Box::new(Type::load(buf)?), false),
            11 => Type::Reference(Box::new(Type::load(buf)?), true),
            12 => Type::Borrow(Box::new(Type::load(buf)?)),
            13 => {
                let mut bytes = [0; 8];
                buf.read_exact(&mut bytes)?;
                let v = u64::from_be_bytes(bytes);
                let ret = Type::load(buf)?;
                let mut vec = Vec::with_capacity(v as usize);
                for _ in 0..v {
                    let t = Type::load(buf)?;
                    buf.read_exact(std::slice::from_mut(&mut c))?;
                    vec.push((t, c != 0));
                }
                Type::Function(Box::new(ret), vec)
            }
            x => panic!("read type value expecting value in 1..=13, got {x}")
        })
    }
}
#[allow(unused_variables)]
pub mod utils;
