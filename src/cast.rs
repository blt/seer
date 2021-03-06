use rustc::ty::{self, Ty};
use syntax::ast::{IntTy, UintTy};
use syntax::ast::FloatTy;

use rustc_apfloat::Float;
use rustc_apfloat::ieee::{Single, Double};
use error::{EvalResult, EvalError};
use eval_context::EvalContext;
use memory::{MemoryPointer, SByte};
use value::PrimVal;

impl<'a, 'tcx> EvalContext<'a, 'tcx> {
    pub(super) fn cast_primval(
        &mut self,
        val: PrimVal,
        src_ty: Ty<'tcx>,
        dest_ty: Ty<'tcx>
    ) -> EvalResult<'tcx, PrimVal> {
        let src_kind = self.ty_to_primval_kind(src_ty)?;

        use value::PrimValKind::*;
        match val {
            PrimVal::Abstract(mut sbytes) => {
                let dest_kind = self.ty_to_primval_kind(dest_ty)?;
                if (src_kind.is_int() || src_kind == Char) && (dest_kind.is_int() || dest_kind == Char) {
                    let src_size = src_kind.num_bytes();
                    let dest_size = dest_kind.num_bytes();
                    for idx in dest_size .. src_size {
                        sbytes[idx] = SByte::Concrete(0);
                    }
                    // TODO(optimization): check to see if the cast has made
                    // the value concrete.
                    Ok(PrimVal::Abstract(sbytes))
                } else if src_kind == Bool && dest_kind.is_int() {
                    let dest_kind = self.ty_to_primval_kind(dest_ty)?;
                    let primval = self.memory.constraints.add_if_then_else(
                        val,
                        dest_kind,
                        PrimVal::Bytes(1),
                        PrimVal::Bytes(0));
                    Ok(primval)
                } else {
                    unimplemented!()
                }
            }
            PrimVal::Undef => Ok(PrimVal::Undef),
            PrimVal::Ptr(ptr) => self.cast_from_ptr(ptr, dest_ty),
            val @ PrimVal::Bytes(_) => {
                use super::PrimValKind::*;
                match src_kind {
                    F32 => unimplemented!(),//self.cast_from_float(val.to_f32()?, dest_ty),
                    F64 => unimplemented!(),//self.cast_from_float(val.to_f64()?, dest_ty),

                    I8 | I16 | I32 | I64 | I128 => {
                        self.cast_from_signed_int(val.to_i128()?, dest_ty)
                    }

                    Bool | Char | U8 | U16 | U32 | U64 | U128 | FnPtr | Ptr => {
                        self.cast_from_int(val.to_u128()?, dest_ty, false)
                    }
                }
            }
        }
    }

    fn cast_from_signed_int(&self, val: i128, ty: ty::Ty<'tcx>) -> EvalResult<'tcx, PrimVal> {
        self.cast_from_int(val as u128, ty, val < 0)
    }

    fn int_to_int(&self, v: i128, ty: IntTy) -> u128 {
        match ty {
            IntTy::I8 => v as i8 as u128,
            IntTy::I16 => v as i16 as u128,
            IntTy::I32 => v as i32 as u128,
            IntTy::I64 => v as i64 as u128,
            IntTy::I128 => v as u128,
            IntTy::Isize => {
                let ty = self.tcx.sess.target.isize_ty;
                self.int_to_int(v, ty)
            }
        }
    }
    fn int_to_uint(&self, v: u128, ty: UintTy) -> u128 {
        match ty {
            UintTy::U8 => v as u8 as u128,
            UintTy::U16 => v as u16 as u128,
            UintTy::U32 => v as u32 as u128,
            UintTy::U64 => v as u64 as u128,
            UintTy::U128 => v,
            UintTy::Usize => {
                let ty = self.tcx.sess.target.usize_ty;
                self.int_to_uint(v, ty)
            }
        }
    }

    fn cast_from_int(
        &self,
        v: u128,
        ty: ty::Ty<'tcx>,
        negative: bool,
    ) -> EvalResult<'tcx, PrimVal> {
        trace!("cast_from_int: {}, {}, {}", v, ty, negative);
        use rustc::ty::TypeVariants::*;
        match ty.sty {
            // Casts to bool are not permitted by rustc, no need to handle them here.
            TyInt(ty) => Ok(PrimVal::Bytes(self.int_to_int(v as i128, ty))),
            TyUint(ty) => Ok(PrimVal::Bytes(self.int_to_uint(v, ty))),

            TyFloat(FloatTy::F32) if negative => Ok(PrimVal::Bytes(Single::from_i128(v as i128).value.to_bits())),
            TyFloat(FloatTy::F64) if negative => Ok(PrimVal::Bytes(Double::from_i128(v as i128).value.to_bits())),
            TyFloat(FloatTy::F32) => Ok(PrimVal::Bytes(Single::from_u128(v as u128).value.to_bits())),
            TyFloat(FloatTy::F64) => Ok(PrimVal::Bytes(Double::from_u128(v as u128).value.to_bits())),

            TyChar if v as u8 as u128 == v => Ok(PrimVal::Bytes(v)),
            TyChar => return Err(EvalError::InvalidChar(v)),

            // No alignment check needed for raw pointers.  But we have to truncate to target ptr size.
            TyRawPtr(_) => Ok(PrimVal::Bytes(self.memory.truncate_to_ptr(v).0 as u128)),

            _ => return Err(EvalError::Unimplemented(format!("int to {:?} cast", ty))),
        }
    }

    fn cast_from_ptr(&self, ptr: MemoryPointer, ty: Ty<'tcx>) -> EvalResult<'tcx, PrimVal> {
        use rustc::ty::TypeVariants::*;
        match ty.sty {
            // Casting to a reference or fn pointer is not permitted by rustc, no need to support it here.
            TyRawPtr(_) |
            TyInt(IntTy::Isize) |
            TyUint(UintTy::Usize) => Ok(PrimVal::Ptr(ptr)),
            TyInt(_) | TyUint(_) => return Err(EvalError::ReadPointerAsBytes),
            _ => return Err(EvalError::Unimplemented(format!("ptr to {:?} cast", ty))),
        }
    }
}
