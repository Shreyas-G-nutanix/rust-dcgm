mod dcgm_bindings;
use dcgm_bindings::*;

// use bindings::*;

// use std::ffi::{CString, CStr};
// use std::os::raw::{c_uint, c_void};
// use std::ptr;
// use std::fmt;
// use std::mem;
// use lazy_static::*;
// use std::collections::HashSet;

// // Global handles and state
// static mut DCGM_LIB_HANDLE: *mut c_void = ptr::null_mut();
// static mut STOP_MODE: Option<Mode> = None;

// #[derive(Debug, Clone, Copy)]
// pub enum Mode {
//     Embedded,
//     Standalone,
//     StartHostengine,
// }

// #[derive(Clone, Debug)]
// pub struct DCGMError {
//     pub message: String,
// }

// impl std::error::Error for DCGMError {}

// impl fmt::Display for DCGMError {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         // Delegate to String’s Display
//         self.message.fmt(f)
//     }
// }

// impl<T: Into<String>> From<T> for DCGMError {
//     fn from(message: T) -> Self {
//         Self {
//             message: message.into(),
//         }
//     }
// }

// unsafe impl Sync for DcgmLib {}

// lazy_static! {
//     static ref DCGM_LIB: Result<DcgmLib, DCGMError> = {
//         let dcgm = unsafe {
//             DcgmLib::new("/lib64/libdcgm.so.4").map_err(|e| {
//                 tracing::error!("Failed to load NVML library: {e}");
//                 DCGMError::from("Failed to load NVML library")
//             })?
//         };
//         Ok(dcgm)
//     };
// }

// pub struct DcgmLibSafe {
//     dcgm: &'static DcgmLib,
//     stop_mode: Mode,
//     handle: dcgmHandle_t
// }

// impl DcgmLibSafe {
//     pub fn new(m: Mode, args: &[&str]) -> Result<Self, DCGMError> {
//         match &*DCGM_LIB {
//             Ok(lib) => {
//                 let mut dcgm = Self {dcgm: lib, stop_mode: m, handle: 0};
//                 dcgm.init();
//                 dcgm.connectToDcgm(m, args);
//                 Ok(dcgm)
//             }
//             Err(err) => Err(err.clone()),
//         }
//     }

//     pub fn init(&mut self) -> Result<(), DCGMError> {

//         match unsafe { self.dcgm.dcgmInit() } {
//             dcgmReturn_enum_DCGM_ST_OK => Ok(()),
//             err_code => Err(DCGMError::from(self.get_error_msg(err_code))),
//         }
//     }

//     pub fn get_error_msg(&self, code: dcgmReturn_t) -> String {
//         let ptr = unsafe { self.dcgm.errorString(code) };
//         if ptr.is_null() {
//             format!("Unknown DCGM error {code}")
//         } else {
//             let cstr = unsafe { CStr::from_ptr(ptr) };
//             cstr.to_string_lossy().into_owned()
//         }
//     }

//     pub fn connectToDcgm(&mut self, m: Mode, args: &[&str]) -> Result<(), DCGMError>{
//         match m{
//             Mode::Embedded => return self.startEmbedded(),
//             Mode::Standalone => return self.connectStandalone(args),
//             Mode::StartHostengine => Err(DCGMError::from("Not implemented")),//return self.startHostengine(),
//             _ => Err(DCGMError::from("Invalid DCGM Mode"))
//         }
//     }

//     pub fn startEmbedded(&mut self) -> Result<(), DCGMError>{
//         match unsafe { self.dcgm.dcgmStartEmbedded(dcgmOperationMode_enum_DCGM_OPERATION_MODE_AUTO, &raw mut self.handle) } {
//             dcgmReturn_enum_DCGM_ST_OK => Ok(()),
//             err_code => Err(DCGMError::from(self.get_error_msg(err_code))),
//         }
//     }

//     pub fn stopEmbedded(&mut self) -> Result<(), DCGMError>{
//         let mut res = match unsafe{self.dcgm.dcgmStopEmbedded(self.handle)}{
//             dcgmReturn_enum_DCGM_ST_OK => Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code))),
//         };
//         res = match unsafe{self.dcgm.dcgmShutdown()}{
//             dcgmReturn_enum_DCGM_ST_OK => Ok(()),
//             err_code => Err(DCGMError::from(self.get_error_msg(err_code))),
//         };
//         return res
//     }

//     pub fn connectStandalone(&mut self, args: &[&str]) -> Result<(), DCGMError>{
//         if args.len() < 2 {
//             return Err(DCGMError::from("missing dcgm address and / or port"))
//         } else{
//             let mut connect_params =  bindings::dcgmConnectV2Params_t{
//                 version: make_version2(std::mem::size_of::<bindings::dcgmConnectV2Params_t>() as u32),
//                 timeoutMs: 3000000,
//                 persistAfterDisconnect: if args.len() == 3 {args[2].parse().unwrap()} else{0},
//                 addressIsUnixSocket: args[1].parse().unwrap()
//             };
//             let addr = CString::new(args[0]).unwrap();
//             match unsafe {self.dcgm.dcgmConnect_v2(addr.as_ptr(), &raw mut connect_params, &raw mut self.handle)}{
//                 dcgmReturn_enum_DCGM_ST_OK => return Ok(()),
//                 err_code => return Err(DCGMError::from(self.get_error_msg(err_code))),
//             };
//         }
//     }

//     pub fn disconnectStandalone(&mut self) -> Result<(), DCGMError>{
//         match unsafe {self.dcgm.dcgmDisconnect(self.handle)}{
//             dcgmReturn_enum_DCGM_ST_OK => (),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         };

//         match unsafe {self.dcgm.dcgmShutdown()}{
//             dcgmReturn_enum_DCGM_ST_OK => return Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         };
//     }

//     pub fn shutdown(&mut self) -> Result<(), DCGMError>{
//         match self.stop_mode{
//             Mode::Embedded => return self.stopEmbedded(),
//             Mode::Standalone => return self.disconnectStandalone(),
//             Mode::StartHostengine => Err(DCGMError::from("Not implemented"))
//         }
//     }

//     pub fn getAllSupportedDevices(&mut self)-> Result<Vec<u32>, DCGMError>{
//         unsafe{
//             let mut gpu_id_list: [std::mem::MaybeUninit<u32>; DCGM_MAX_NUM_DEVICES as usize] = 
//                 std::mem::MaybeUninit::uninit().assume_init();
//             let mut count: std::mem::MaybeUninit<i32> = std::mem::MaybeUninit::uninit().assume_init();
//             let res = self.dcgm.dcgmGetAllSupportedDevices(
//                 self.handle, 
//                 gpu_id_list.as_mut_ptr() as *mut std::os::raw::c_uint, 
//                 count.as_mut_ptr());
//             if res != dcgmReturn_enum_DCGM_ST_OK{
//                 return Err(DCGMError::from(self.get_error_msg(res)))
//             }
//             let initialized_slice = std::slice::from_raw_parts(gpu_id_list.as_mut_ptr() as *mut u32, count.assume_init() as usize);
//             return Ok(initialized_slice.to_vec())
//         }
//     }

//     pub fn createGroup(&mut self, group_name: &String) -> Result<dcgmGpuGrp_t, DCGMError>{
//         let mut groupId: dcgmGpuGrp_t = 0;
//         match unsafe{self.dcgm.dcgmGroupCreate(
//             self.handle, 
//             dcgmGroupType_enum_DCGM_GROUP_EMPTY,
//             CString::new(group_name.clone()).unwrap().as_ptr(), 
//             &raw mut groupId)}{

//             dcgmReturn_enum_DCGM_ST_OK => return Ok(groupId),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         };
//     }

//     pub fn addEntityToGroup(&mut self, groupId: dcgmGpuGrp_t, entityGroupID: dcgm_field_entity_group_t, entityId: u32)->Result<(), DCGMError>{
//         match unsafe{self.dcgm.dcgmGroupAddEntity(
//             self.handle,
//             groupId,
//             entityGroupID,
//             entityId
//         )}{
//             dcgmReturn_enum_DCGM_ST_OK => return Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn destroyGroup(&mut self, groupId: dcgmGpuGrp_t)->Result<(), DCGMError>{
//         match unsafe{self.dcgm.dcgmGroupDestroy(self.handle, groupId)}{
//             dcgmReturn_enum_DCGM_ST_OK => return Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn fieldGroupCreate(&mut self, fieldGroupName: &String, fieldIds: &mut [u16])-> Result<dcgmFieldGrp_t, DCGMError>{
//         let mut fieldHandle: dcgmFieldGrp_t = 0;
//         match unsafe{self.dcgm.dcgmFieldGroupCreate(
//             self.handle, fieldIds.len() as i32, 
//             fieldIds.as_mut_ptr(), 
//             CString::new(fieldGroupName.clone()).unwrap().as_ptr(), 
//             &raw mut fieldHandle)}{

//             dcgmReturn_enum_DCGM_ST_OK => return Ok(fieldHandle),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn fieldGroupDestroy(&mut self, dcgmFieldGroupId: dcgmFieldGrp_t)->Result<(), DCGMError>{
//         match unsafe{self.dcgm.dcgmFieldGroupDestroy(self.handle, dcgmFieldGroupId)}{
//             dcgmReturn_enum_DCGM_ST_OK => return Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn watchFields(&mut self, fieldGroupId: dcgmFieldGrp_t, groupId: dcgmGpuGrp_t, updateFreq: i64, maxKeepAge: f64, maxKeepSamples: i32)->Result<(), DCGMError>{
//         match unsafe{self.dcgm.dcgmWatchFields(self.handle, groupId, fieldGroupId, updateFreq, maxKeepAge, maxKeepSamples)}{
//             dcgmReturn_enum_DCGM_ST_OK => (),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         };
//         return self.updateAllFields();
//     }

//     pub fn updateAllFields(&mut self)->Result<(), DCGMError>{
//         match unsafe{self.dcgm.dcgmUpdateAllFields(self.handle, 1)}{
//             dcgmReturn_enum_DCGM_ST_OK => Ok(()),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn entitiesGetLatestValues(&mut self, entities: &mut[dcgmGroupEntityPair_t], fields: &mut[u16], flags: u32) -> Result<Vec<dcgmFieldValue_v2>, DCGMError>{
//         let mut values = Vec::<dcgmFieldValue_v2>::with_capacity(fields.len()*entities.len());
//         match unsafe{self.dcgm.dcgmEntitiesGetLatestValues(
//             self.handle, 
//             &mut entities[0], 
//             entities.len() as c_uint, 
//             &mut fields[0],
//             fields.len() as c_uint,
//             flags,
//             &mut values[0])}{

//             dcgmReturn_enum_DCGM_ST_OK => Ok(values),
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn selectGpusByTopology(&mut self, gpuIds: &HashSet<u32>, numGpus: u32) -> Result<HashSet<u32>, DCGMError>{
//         let mut gpuBitmask: u64 = 0;
//         for gpu in gpuIds{
//             if *gpu > 63 {
//                 return Err(DCGMError::from("gpu value out of bounds"));
//             }
//             gpuBitmask |= 1 << *gpu;
//         }
//         let mut outputBitmask: u64 = 0;
//         match unsafe{self.dcgm.dcgmSelectGpusByTopology(
//             self.handle,
//             gpuBitmask,
//             numGpus,
//             &raw mut outputBitmask,
//             0
//         )}{
//             dcgmReturn_enum_DCGM_ST_OK => 
//                 { 
//                     let mut indices = HashSet::<u32>::with_capacity(numGpus as usize);
//                     let mut index: u32 = 0;
//                     while outputBitmask != 0{
//                     if outputBitmask & 1 == 1{
//                         indices.insert(index);
//                     }
//                     outputBitmask >>= 1;
//                     index += 1;
//                 } return Ok(indices)},
//             err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//         }
//     }

//     pub fn getNvLinkLinkStatus(&mut self) -> Result<Vec<NvLinkStatus>, DCGMError>{
//         unsafe{
//             let mut linkStatus: dcgmNvLinkStatus_t = std::mem::MaybeUninit::uninit().assume_init();
//             linkStatus.version = make_version4(std::mem::size_of::<dcgmNvLinkStatus_t>() as u32);
//             match self.dcgm.dcgmGetNvLinkLinkStatus(self.handle, &raw mut linkStatus){
//                 dcgmReturn_enum_DCGM_ST_OK => (),
//                 err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//             }
//             let mut statuses = Vec::<NvLinkStatus>::with_capacity((linkStatus.numGpus*DCGM_NVLINK_MAX_LINKS_PER_GPU+linkStatus.numNvSwitches*DCGM_NVLINK_MAX_LINKS_PER_NVSWITCH) as usize);
//             let mut index = 0;
//             for i in 0..linkStatus.numGpus{
//                 for j in 0..DCGM_NVLINK_MAX_LINKS_PER_GPU{
//                     let link = NvLinkStatus{
//                         parent_id: linkStatus.gpus[i as usize].entityId,
//                         parent_type: dcgm_field_entity_group_t_DCGM_FE_GPU,
//                         state: linkStatus.gpus[i as usize].linkState[j as usize],
//                         index: j
//                     };

//                     statuses.push(link);
//                     index += 1;
//                 }
//             }

//             for i in 0..linkStatus.numNvSwitches{
//                 for j in 0..DCGM_NVLINK_MAX_LINKS_PER_NVSWITCH{
//                     let link = NvLinkStatus{
//                         parent_id: linkStatus.gpus[i as usize].entityId,
//                         parent_type: dcgm_field_entity_group_t_DCGM_FE_SWITCH,
//                         state: linkStatus.gpus[i as usize].linkState[j as usize],
//                         index: j
//                     };

//                     statuses.push(link);
//                     index += 1;
//                 }
//             }
//             return Ok(statuses);
//         }
//     }

//     pub fn getDeviceAttributes(&mut self, gpuId: u32) -> Result<dcgmDeviceAttributes_t, DCGMError>{
//         unsafe{
//             let mut device: dcgmDeviceAttributes_t = std::mem::MaybeUninit::uninit().assume_init();
//             match self.dcgm.dcgmGetDeviceAttributes(self.handle, gpuId, &mut device){
//                 dcgmReturn_enum_DCGM_ST_OK => Ok(device),
//                 err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//             }
//         }
//     }

//     pub fn getDeviceTopology(&mut self, gpuId: u32) -> Result<Vec<P2PLink>, DCGMError>{
//         unsafe{
//             let mut topology: dcgmDeviceTopology_t = std::mem::MaybeUninit::uninit().assume_init();
//             match self.dcgm.dcgmGetDeviceTopology(self.handle, gpuId, &mut topology){
//                 dcgmReturn_enum_DCGM_ST_OK => (),
//                 err_code => return Err(DCGMError::from(self.get_error_msg(err_code)))
//             };
//             let device = self.getDeviceAttributes(gpuId)?;
//             let mut links = Vec::<P2PLink>::with_capacity(topology.numGpus as usize);
//             for i in 0..topology.numGpus{
//                 let link = P2PLink{
//                     gpu : topology.gpuPaths[i as usize].gpuId,
//                     bus_id: String::from_utf8(Vec::<u8>::from_iter((device.identifiers.pciBusId).iter().take_while(|&&c| c != 0).map(|&c| c as u8).collect())).unwrap(),
//                     link: topology.gpuPaths[i as usize].path
//                 };
//                 links.push(link);
//             }
//             return Ok(links);
//         }
//     }
            
// }

// pub fn dereference_field_value_v2(fv: &dcgmFieldValue_v2) -> Result<String, DCGMError> {
//     match fv.status{
//         dcgmReturn_enum_DCGM_ST_OK => (),
//         dcgmReturn_enum_DCGM_ST_NOT_WATCHED => return Err(DCGMError::from("Field Value is not being watched")),
//         _ => return Err(DCGMError::from("Unknown or Unimplemented Return Status"))
//     };
//     return Ok("a".to_string());
// }

// struct NvLinkStatus{
//     parent_id: u32,
//     parent_type: dcgm_field_entity_group_t,
//     state: dcgmNvLinkLinkState_t,
//     index: u32,
// }

// struct P2PLink{
//     gpu: u32,
//     bus_id: String,
//     link: dcgmGpuLevel_enum
// }

// fn make_version1(struct_type: u32) -> u32 {
//     struct_type | (1 << 24)
// }

// fn make_version2(struct_type: u32) -> u32 {
//     struct_type | (2 << 24)
// }

// fn make_version4(struct_type: u32) -> u32 {
// 	struct_type | 4<<24
// }

fn main() {
    
    let mode = Mode::Standalone;
    let args = ["127.0.0.1:5555", "0", "1"];
    let mut dcgm = DcgmLibSafe::new(mode, &args).unwrap();
   
    println!("Devices: {:?}", dcgm.getAllSupportedDevices().unwrap())
    
}
