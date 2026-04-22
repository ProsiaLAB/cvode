#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::inline_always)]

use std::ffi::{CString, c_void};

use sundials_sys::{
    CV_BDF, CV_NORMAL, CV_ONE_STEP, FILE, SUN_COMM_NULL, SUNOutputFormat_SUN_OUTPUTFORMAT_TABLE,
    SUNPrecType_SUN_PREC_LEFT,
};
use sundials_sys::{
    CVLsPrecSetupFn, CVLsPrecSolveFn, CVRhsFn, N_Vector, SUNContext, SUNLinearSolver,
};
use sundials_sys::{
    CVode, CVodeCreate, CVodeFree, CVodeGetCurrentStep, CVodeGetCurrentTime, CVodeGetLastStep,
    CVodeInit, CVodePrintAllStats, CVodeReInit, CVodeSVtolerances, CVodeSetConstraints,
    CVodeSetEpsLin, CVodeSetEtaConvFail, CVodeSetInitStep, CVodeSetJacEvalFrequency,
    CVodeSetLSetupFrequency, CVodeSetLinearSolver, CVodeSetMaxConvFails, CVodeSetMaxErrTestFails,
    CVodeSetMaxHnilWarns, CVodeSetMaxNonlinIters, CVodeSetMaxNumConstraintFails,
    CVodeSetMaxNumSteps, CVodeSetMaxOrd, CVodeSetMaxStep, CVodeSetMinStep, CVodeSetPreconditioner,
    CVodeSetStabLimDet, CVodeSetUserData, N_VClone, N_VDestroy_Serial, N_VGetArrayPointer_Serial,
    N_VNew_Serial, N_VScale, SUNContext_Create, SUNContext_Free, SUNLinSol_SPBCGS,
    SUNLinSol_SPFGMR, SUNLinSol_SPGMR, SUNLinSol_SPTFQMR, SUNLinSolFree, fclose, fopen,
};

#[derive(Default)]
pub struct SunContext {
    ptr: SUNContext,
}

impl SunContext {
    /// Create a new [`SunContext`].
    ///
    /// # Panics
    /// Panics if the context creation fails (i.e., if the returned pointer is null
    /// which should not happen under normal circumstances).
    #[must_use]
    #[inline(always)]
    pub fn new() -> Self {
        let mut ctx = std::ptr::null_mut();
        unsafe { SUNContext_Create(SUN_COMM_NULL, &raw mut ctx) };
        assert!(!ctx.is_null());
        Self { ptr: ctx }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_raw(&self) -> SUNContext {
        self.ptr
    }
}

impl Drop for SunContext {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { SUNContext_Free(&raw mut self.ptr) };
    }
}

pub struct NVector {
    ptr: N_Vector,
    len: usize,
}

impl NVector {
    /// Create a new serial [`NVector`] of the given length, associated with the provided [`SunContext`].
    ///
    /// # Panics
    /// Panics if the vector creation fails (i.e., if the returned pointer is null
    /// which can happen if the length is zero or if there is an issue with the context).
    #[must_use]
    #[inline(always)]
    pub fn new_serial(len: usize, ctx: &SunContext) -> Self {
        let ptr = unsafe { N_VNew_Serial(len as i64, ctx.as_raw()) };
        assert!(!ptr.is_null());
        Self { ptr, len }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_raw(&self) -> N_Vector {
        self.ptr
    }

    #[must_use]
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.len
    }

    #[must_use]
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[must_use]
    #[inline(always)]
    pub fn slice(&self) -> &[f64] {
        let p = unsafe { N_VGetArrayPointer_Serial(self.ptr) };
        unsafe { std::slice::from_raw_parts(p, self.len) }
    }

    #[inline(always)]
    pub fn slice_mut(&mut self) -> &mut [f64] {
        let p = unsafe { N_VGetArrayPointer_Serial(self.ptr) };
        unsafe { std::slice::from_raw_parts_mut(p, self.len) }
    }
}

impl Drop for NVector {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { N_VDestroy_Serial(self.ptr) };
    }
}

impl Clone for NVector {
    fn clone(&self) -> Self {
        unsafe {
            let new = N_VClone(self.ptr);
            N_VScale(1.0, self.ptr, new);

            Self {
                ptr: new,
                len: self.len,
            }
        }
    }
}

pub struct LinearSolver {
    ptr: SUNLinearSolver,
}

impl LinearSolver {
    /// Create a new SPGMR linear solver for the given vector and context.
    ///
    /// # Panics
    /// Panics if the solver creation fails (i.e., if the returned pointer is null
    /// which can happen if there is an issue with the vector or context).
    #[must_use]
    #[inline(always)]
    pub fn spgmr(y: &NVector, ctx: &SunContext) -> Self {
        let solver = unsafe {
            SUNLinSol_SPGMR(
                y.as_raw(),
                SUNPrecType_SUN_PREC_LEFT.try_into().unwrap(),
                0,
                ctx.as_raw(),
            )
        };
        assert!(!solver.is_null());
        Self { ptr: solver }
    }

    /// Create a new SPFGMR linear solver for the given vector and context.
    ///
    /// # Panics
    /// Panics if the solver creation fails (i.e., if the returned pointer is null
    /// which can happen if there is an issue with the vector or context).
    #[must_use]
    #[inline(always)]
    pub fn spfgmr(y: &NVector, ctx: &SunContext) -> Self {
        let solver = unsafe {
            SUNLinSol_SPFGMR(
                y.as_raw(),
                SUNPrecType_SUN_PREC_LEFT.try_into().unwrap(),
                0,
                ctx.as_raw(),
            )
        };
        assert!(!solver.is_null());
        Self { ptr: solver }
    }

    /// Create a new SPBCGS linear solver for the given vector and context.
    ///
    /// # Panics
    /// Panics if the solver creation fails (i.e., if the returned pointer is null
    /// which can happen if there is an issue with the vector or context).
    #[must_use]
    #[inline(always)]
    pub fn spbcgs(y: &NVector, ctx: &SunContext) -> Self {
        let solver = unsafe {
            SUNLinSol_SPBCGS(
                y.as_raw(),
                SUNPrecType_SUN_PREC_LEFT.try_into().unwrap(),
                0,
                ctx.as_raw(),
            )
        };
        assert!(!solver.is_null());
        Self { ptr: solver }
    }

    /// Create a new SPTFQMR linear solver for the given vector and context.
    ///
    /// # Panics
    /// Panics if the solver creation fails (i.e., if the returned pointer is null
    /// which can happen if there is an issue with the vector or context).
    #[must_use]
    #[inline(always)]
    pub fn sptfqmr(y: &NVector, ctx: &SunContext) -> Self {
        let solver = unsafe {
            SUNLinSol_SPTFQMR(
                y.as_raw(),
                SUNPrecType_SUN_PREC_LEFT.try_into().unwrap(),
                0,
                ctx.as_raw(),
            )
        };
        assert!(!solver.is_null());
        Self { ptr: solver }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_raw(&self) -> SUNLinearSolver {
        self.ptr
    }
}

impl Drop for LinearSolver {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { SUNLinSolFree(self.ptr) };
    }
}

pub struct Cvode {
    ptr: *mut c_void,
}

impl Cvode {
    /// Create a new CVODE solver in BDF mode with the given context.
    ///
    /// # Panics
    /// Panics if the solver creation fails (i.e., if the returned pointer is null
    /// which can happen if there is an issue with the context).
    #[must_use]
    #[inline(always)]
    pub fn new_bdf(ctx: &SunContext) -> Self {
        let p = unsafe { CVodeCreate(CV_BDF, ctx.as_raw()) };
        assert!(!p.is_null());
        Self { ptr: p }
    }

    #[must_use]
    #[inline(always)]
    pub fn as_raw(&self) -> *mut c_void {
        self.ptr
    }

    #[inline(always)]
    pub fn set_userdata<T>(&self, data: &mut T) {
        unsafe {
            CVodeSetUserData(self.ptr, std::ptr::from_mut::<T>(data).cast::<c_void>());
        }
    }

    #[inline(always)]
    pub fn init(&self, rhs: CVRhsFn, t0: f64, y: &NVector) {
        unsafe { CVodeInit(self.ptr, rhs, t0, y.as_raw()) };
    }

    #[inline(always)]
    pub fn reinit(&self, t0: f64, y: &NVector) {
        unsafe { CVodeReInit(self.ptr, t0, y.as_raw()) };
    }

    #[inline(always)]
    pub fn set_tolerances(&self, reltol: f64, abstol: &NVector) {
        unsafe { CVodeSVtolerances(self.ptr, reltol, abstol.as_raw()) };
    }

    #[inline(always)]
    pub fn set_linear_solver(&self, linsol: &LinearSolver) {
        unsafe { CVodeSetLinearSolver(self.ptr, linsol.as_raw(), std::ptr::null_mut()) };
    }

    #[inline(always)]
    pub fn integrate(&self, tout: f64, y: &NVector, t: &mut f64) -> i32 {
        unsafe { CVode(self.ptr, tout, y.as_raw(), t, CV_NORMAL) }
    }

    #[inline(always)]
    pub fn integrate_one_step(&self, tout: f64, y: &NVector, t: &mut f64) -> i32 {
        unsafe { CVode(self.ptr, tout, y.as_raw(), t, CV_ONE_STEP) }
    }

    #[must_use]
    #[inline(always)]
    pub fn get_current_time(&self) -> f64 {
        let mut t = 0.0;
        unsafe { CVodeGetCurrentTime(self.ptr, &raw mut t) };
        t
    }

    #[must_use]
    #[inline(always)]
    pub fn get_last_step(&self) -> f64 {
        let mut h = 0.0;
        unsafe { CVodeGetLastStep(self.ptr, &raw mut h) };
        h
    }

    #[must_use]
    #[inline(always)]
    pub fn get_current_step(&self) -> f64 {
        let mut h = 0.0;
        unsafe { CVodeGetCurrentStep(self.ptr, &raw mut h) };
        h
    }

    #[inline(always)]
    pub fn set_preconditioner(&self, setup: CVLsPrecSetupFn, solve: CVLsPrecSolveFn) {
        unsafe {
            CVodeSetPreconditioner(self.ptr, setup, solve);
        }
    }

    #[inline(always)]
    pub fn set_constraints(&self, constraints: &NVector) {
        unsafe { CVodeSetConstraints(self.ptr, constraints.as_raw()) };
    }

    #[inline(always)]
    pub fn set_max_nonlin_iters(&self, n: i32) {
        unsafe { CVodeSetMaxNonlinIters(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_max_conv_fails(&self, n: i32) {
        unsafe { CVodeSetMaxConvFails(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_eta_conv_fail(&self, eta: f64) {
        unsafe { CVodeSetEtaConvFail(self.ptr, eta) };
    }

    #[inline(always)]
    pub fn set_max_err_test_fails(&self, n: i32) {
        unsafe { CVodeSetMaxErrTestFails(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_max_constraints_fails(&self, n: i32) {
        unsafe { CVodeSetMaxNumConstraintFails(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_init_step(&self, h0: f64) {
        unsafe { CVodeSetInitStep(self.ptr, h0) };
    }

    #[inline(always)]
    pub fn set_min_step(&self, hmin: f64) {
        unsafe { CVodeSetMinStep(self.ptr, hmin) };
    }

    #[inline(always)]
    pub fn set_max_step(&self, hmax: f64) {
        unsafe { CVodeSetMaxStep(self.ptr, hmax) };
    }

    #[inline(always)]
    pub fn set_max_ord(&self, ord: i32) {
        unsafe { CVodeSetMaxOrd(self.ptr, ord) };
    }

    #[inline(always)]
    pub fn set_max_num_steps(&self, n: i64) {
        unsafe { CVodeSetMaxNumSteps(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_max_hnil_warns(&self, n: i32) {
        unsafe { CVodeSetMaxHnilWarns(self.ptr, n) };
    }

    #[inline(always)]
    pub fn set_stability_limit_detection(&self, enable: i32) {
        unsafe { CVodeSetStabLimDet(self.ptr, enable) };
    }

    #[inline(always)]
    pub fn set_jac_eval_frequency(&self, freq: i64) {
        unsafe { CVodeSetJacEvalFrequency(self.ptr, freq) };
    }

    #[inline(always)]
    pub fn set_linear_solver_setup_frequency(&self, freq: i64) {
        unsafe { CVodeSetLSetupFrequency(self.ptr, freq) };
    }

    #[inline(always)]
    pub fn set_epslin(&self, eps: f64) {
        unsafe { CVodeSetEpsLin(self.ptr, eps) };
    }

    /// Save CVODE statistics to a file named ``cvode_stats.txt`` in a human-readable table format.
    ///
    /// # Panics
    /// Panics if the file cannot be opened for writing, which can happen due to permission
    /// issues or if the filesystem is read-only.
    /// Panics if the CVODE statistics cannot be printed, which can happen if there is an internal error in the CVODE library.
    /// Note that this function will overwrite the file ``cvode_stats.txt`` if it already exists.
    #[inline(always)]
    pub fn save_statistics(&self) {
        unsafe {
            let filename = CString::new("cvode_stats.txt").unwrap();
            let mode = CString::new("w").unwrap();

            let file: *mut FILE = fopen(filename.as_ptr(), mode.as_ptr());
            assert!(!file.is_null(), "Failed to open file");

            let retval = CVodePrintAllStats(self.ptr, file, SUNOutputFormat_SUN_OUTPUTFORMAT_TABLE);
            if retval != 0 {
                eprintln!("Failed to print CVode statistics");
            }
            fclose(file);
        };
    }
}

impl Drop for Cvode {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe { CVodeFree(&raw mut self.ptr) };
    }
}
