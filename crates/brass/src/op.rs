use num_enum::{IntoPrimitive, TryFromPrimitive};

extern crate strum;

use strum_macros::IntoStaticStr;

// Copyright 2022 Zenturi Software Co.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[derive(IntoPrimitive, TryFromPrimitive, IntoStaticStr)]
#[repr(u8)]
pub enum Op {
    OMov = 0,
    OInt = 1,
    OFloat = 2,
    OBool = 3,
    OBytes = 4,
    OString = 5,
    ONull = 6,

    OAdd = 7,
    OSub = 8,
    OMul = 9,
    OSDiv = 10,
    OUDiv = 11,
    OSMod = 12,
    OUMod = 13,
    OShl = 14,
    OSShr = 15,
    OUShr = 16,
    OAnd = 17,
    OOr = 18,
    OXor = 19,

    ONeg = 20,
    ONot = 21,
    OIncr = 22,
    ODecr = 23,

    OCall0 = 24,
    OCall1 = 25,
    OCall2 = 26,
    OCall3 = 27,
    OCall4 = 28,
    OCallN = 29,
    OCallMethod = 30,
    OCallThis = 31,
    OCallClosure = 32,

    OStaticClosure = 33,
    OInstanceClosure = 34,
    OVirtualClosure = 35,

    OGetGlobal = 36,
    OSetGlobal = 37,
    OField = 38,
    OSetField = 39,
    OGetThis = 40,
    OSetThis = 41,
    ODynGet = 42,
    ODynSet = 43,

    OJTrue = 44,
    OJFalse = 45,
    OJNull = 46,
    OJNotNull = 47,
    OJSLt = 48,
    OJSGte = 49,
    OJSGt = 50,
    OJSLte = 51,
    OJULt = 52,
    OJUGte = 53,
    OJNotLt = 54,
    OJNotGte = 55,
    OJEq = 56,
    OJNotEq = 57,
    OJAlways = 58,

    OToDyn = 59,
    OToSFloat = 60,
    OToUFloat = 61,
    OToInt = 62,
    OSafeCast = 63,
    OUnsafeCast = 64,
    OToVirtual = 65,

    OLabel = 66,
    ORet = 67,
    OThrow = 68,
    ORethrow = 69,
    OSwitch = 70,
    ONullCheck = 71,
    OTrap = 72,
    OEndTrap = 73,

    OGetI8 = 74,
    OGetI16 = 75,
    OGetMem = 76,
    OGetArray = 77,
    OSetI8 = 78,
    OSetI16 = 79,
    OSetMem = 80,
    OSetArray = 81,

    ONew = 82,
    OArraySize = 83,
    OType = 84,
    OGetType = 85,
    OGetTID = 86,

    ORef = 87,
    OUnref = 88,
    OSetref = 89,

    OMakeEnum = 90,
    OEnumAlloc = 91,
    OEnumIndex = 92,
    OEnumField = 93,
    OSetEnumField = 94,

    OAssert = 95,
    ORefData = 96,
    ORefOffset = 97,
    ONop = 98,

    OLast = 99,
}

pub static OP_NARGS: [i8; 100] = [
    2, 2, 2, 2, 2, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 1, 1, 2, 3, 4, 5, 6, -1, -1,
    -1, -1, 2, 3, 3, 2, 2, 3, 3, 2, 2, 3, 3, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2,
    2, 2, 2, 2, 0, 1, 1, 1, -1, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2, 2, 2, 2, 2, -1, 2, 2,
    4, 3, 0, 2, 3, 0, 0,
];
