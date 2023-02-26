#![allow(clippy::not_unsafe_ptr_arg_deref)]

//! conconcommand related abstractions

use super::northstar::CREATE_OBJECT_FUNC;
use std::ffi::CStr;
use std::os::raw::c_void;
use std::ptr;

use crate::bindings::command::{CCommand, ConCommand, ConCommandConstructorType};
use crate::bindings::plugin_abi::ObjectType_CONCOMMANDS;
use crate::to_sq_string;

use super::errors::RegisterError;

/// [`CCommandResult`] gets all the usefull stuff from [`*const CCommand`] and puts in this safe struct 
#[derive(Debug, Default)]
pub struct CCommandResult {
    pub args: Vec<String>,
    pub command: String,
}

impl From<*const CCommand> for CCommandResult {
    fn from(value: *const CCommand) -> Self {
        let ccommand = match unsafe { value.as_ref() } {
            Some(c) => c,
            None => return Self::default(),
        };
        let ccommand = *ccommand;

        let (args, command) = unsafe {
            if ccommand.m_nArgv0Size == 0 {
                (Vec::new(), "".to_string())
            } else {
                let whole_command = CStr::from_ptr(&ccommand.m_pArgSBuffer[0]).to_string_lossy().to_string();
                let mut whole_command = whole_command.split_whitespace();

                let command = whole_command.next().unwrap_or_default().into();
                let args = whole_command.map(|a| a.to_string()).collect();

                (args, command)
            }
        };

        Self { args, command }
    }
}

pub struct RegisterConCommands {
    reg_func: ConCommandConstructorType,
}

impl RegisterConCommands {
    pub(crate) unsafe fn new(ptr: *const c_void) -> Self {
        let reg_func: ConCommandConstructorType = std::mem::transmute(ptr);

        Self { reg_func }
    }

    pub fn register_concommand(
        &self,
        name: String,
        callback: extern "C" fn(arg1: *const CCommand),
        help_string: String,
        flags: i32,
    ) -> Result<(), RegisterError> {
        let name = to_sq_string!(name);
        let name_ptr = name.as_ptr();

        let help_string = to_sq_string!(help_string);
        let help_string_ptr = help_string.as_ptr();

        let command: *mut ConCommand = unsafe {
            std::mem::transmute((CREATE_OBJECT_FUNC
                .wait()
                .ok_or(RegisterError::NoneFunction)?)(
                ObjectType_CONCOMMANDS
            ))
        };

        unsafe {
            self.reg_func.unwrap()(
                command,
                name_ptr,
                Some(callback),
                help_string_ptr,
                flags,
                ptr::null_mut(),
            )
        };
        Ok(())
    }
}