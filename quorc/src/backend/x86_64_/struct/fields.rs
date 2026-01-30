use crate::backend::x86_64_::{align_up, size_align_of};
use crate::frontend::ast::Type;

pub fn layout_of_struct(instances: &[Type]) -> (usize, usize) {
    if instances.is_empty() {
        return (0, 1);
    }
    let mut off = 0usize;
    let mut max_a = 1usize;
    for ty in instances {
        let (sz, al) = size_align_of(ty);
        let al = al.max(1);
        off = align_up(off, al);
        off += sz;
        max_a = max_a.max(al);
    }
    let size = align_up(off, max_a);
    (size, max_a)
}

#[derive(Debug)]
pub struct FieldLayout {
    pub name: String,
    pub offset: usize,
}

pub struct StructLayout {
    pub _size: usize,
    pub _align: usize,
    pub fields: Vec<FieldLayout>,
}

pub fn layout_fields(fields: &[(String, Type)]) -> StructLayout {
    let mut off = 0usize;
    let mut max_a = 1usize;
    let mut out = Vec::with_capacity(fields.len());

    for (name, ty) in fields {
        let (sz, al) = size_align_of(ty);
        let al = al.max(1);
        off = align_up(off, al);
        out.push(FieldLayout {
            name: name.clone(),
            offset: off,
        });
        off += sz;
        max_a = max_a.max(al);
    }

    let size = align_up(off, max_a);

    StructLayout {
        _size: size,
        _align: max_a,
        fields: out,
    }
}
