use wasmer_runtime::Ctx;

#[inline(always)]
pub fn pointer_to_str(ctx: &mut Ctx, ptr: u32, len: u32) -> String {
    let memory = ctx.memory(0);

    // Get a subslice that corresponds to the memory used by the string.
    let str_vec: Vec<_> = memory.view()[ptr as usize..(ptr + len) as usize]
        .iter()
        .map(|cell| cell.get())
        .collect();
    
    let str_slice = &str_vec;

    // TODO find a non allocating way :(
    std::str::from_utf8(str_slice).unwrap().to_owned()
}
