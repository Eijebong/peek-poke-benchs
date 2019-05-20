use peek_poke::PeekPoke;
use std::{fmt::Debug, marker::PhantomData};

fn poke_into<V: PeekPoke>(a: &V) -> Vec<u8> {
    let mut v = <Vec<u8>>::with_capacity(<V>::max_size());
    let end_ptr = a.poke_into(v.as_mut_ptr());
    let new_size = end_ptr as usize - v.as_ptr() as usize;
    assert!(new_size <= v.capacity());
    unsafe {
        v.set_len(new_size);
    }
    v
}

fn the_same<V>(a: V)
where
    V: PartialEq + Debug + PeekPoke,
{
    let v = poke_into(&a);
    let mut b: V = unsafe { std::mem::uninitialized() };
    let end_ptr = b.peek_from(v.as_ptr());
    let size = end_ptr as usize - v.as_ptr() as usize;
    assert_eq!(size, v.len());
    assert_eq!(a, b);
}

#[test]
fn test_numbers() {
    // unsigned positive
    the_same(5u8);
    the_same(5u16);
    the_same(5u32);
    the_same(5u64);
    the_same(5usize);
    // signed positive
    the_same(5i8);
    the_same(5i16);
    the_same(5i32);
    the_same(5i64);
    the_same(5isize);
    // signed negative
    the_same(-5i8);
    the_same(-5i16);
    the_same(-5i32);
    the_same(-5i64);
    the_same(-5isize);
    // floating
    the_same(-100f32);
    the_same(0f32);
    the_same(5f32);
    the_same(-100f64);
    the_same(5f64);
}

#[test]
fn test_bool() {
    the_same(true);
    the_same(false);
}

#[test]
fn test_option() {
    the_same(Some(5usize));
    //the_same(Some("foo bar".to_string()));
    the_same(None::<usize>);
}

/*
#[test]
fn test_fixed_size_array() {
    the_same([24u32; 32]);
    the_same([1u64, 2, 3, 4, 5, 6, 7, 8]);
    the_same([0u8; 19]);
}
 */

#[test]
fn test_tuple() {
    the_same((1isize,));
    the_same((1isize, 2isize, 3isize));
    the_same((1isize, ()));
}

#[test]
fn test_basic_struct() {
    #[derive(PartialEq, Debug, PeekPoke)]
    struct Bar {
        a: u32,
        b: u32,
        c: u32,
        d: Option<u32>,
    }

    the_same(Bar {
        a: 2,
        b: 4,
        c: 42,
        d: None,
    });
}

#[test]
fn test_enum() {
    #[derive(PartialEq, Debug, PeekPoke)]
    enum TestEnum {
        NoArg,
        OneArg(usize),
        Args(usize, usize),
        AnotherNoArg,
        StructLike { x: usize, y: f32 },
    }
    the_same(TestEnum::NoArg);
    the_same(TestEnum::OneArg(4));
    the_same(TestEnum::Args(4, 5));
    the_same(TestEnum::AnotherNoArg);
    the_same(TestEnum::StructLike { x: 4, y: 3.14159 });
}

#[test]
fn test_enum_cstyle() {
    #[repr(u32)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PeekPoke)]
    enum BorderStyle {
        None = 0,
        Solid = 1,
        Double = 2,
        Dotted = 3,
        Dashed = 4,
        Hidden = 5,
        Groove = 6,
        Ridge = 7,
        Inset = 8,
        Outset = 9,
    }
    the_same(BorderStyle::None);
    the_same(BorderStyle::Solid);
    the_same(BorderStyle::Double);
    the_same(BorderStyle::Dotted);
    the_same(BorderStyle::Dashed);
    the_same(BorderStyle::Hidden);
    the_same(BorderStyle::Groove);
    the_same(BorderStyle::Ridge);
    the_same(BorderStyle::Inset);
    the_same(BorderStyle::Outset);
}

#[test]
fn test_phantom_data() {
    struct Bar;
    #[derive(Debug, PartialEq, Eq, PeekPoke)]
    struct Foo {
        x: u32,
        y: u32,
        _marker: PhantomData<Bar>,
    }
    the_same(Foo {
        x: 19,
        y: 42,
        _marker: PhantomData,
    });
}

#[test]
fn test_generic() {
    #[derive(Debug, PartialEq, Eq, PeekPoke)]
    struct Foo<T> {
        x: T,
        y: T,
    }
    the_same(Foo { x: 19.0, y: 42.0 });
}

#[cfg(feature = "extras")]
mod extra_tests {
    use super::*;
    use euclid::{Point2D, Rect, SideOffsets2D, Size2D, Transform3D, Vector2D};
    use std::mem::size_of;

    #[test]
    fn euclid_types() {
        the_same(Point2D::<f32>::new(1.0, 2.0));
        assert_eq!(Point2D::<f32>::max_size(), 2 * size_of::<f32>());

        the_same(Rect::<f32>::new(
            Point2D::<f32>::new(0.0, 0.0),
            Size2D::<f32>::new(100.0, 80.0),
        ));
        assert_eq!(Rect::<f32>::max_size(), 4 * size_of::<f32>());

        the_same(SideOffsets2D::<f32>::new(0.0, 10.0, -1.0, -10.0));
        assert_eq!(SideOffsets2D::<f32>::max_size(), 4 * size_of::<f32>());

        the_same(Transform3D::<f32>::identity());
        assert_eq!(Transform3D::<f32>::max_size(), 16 * size_of::<f32>());

        the_same(Vector2D::<f32>::new(1.0, 2.0));
        assert_eq!(Vector2D::<f32>::max_size(), 2 * size_of::<f32>());
    }

    #[test]
    fn webrender_api_types() {
        type PipelineSourceId = i32;
        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct PipelineId(pub PipelineSourceId, pub u32);

        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct ClipChainId(pub u64, pub PipelineId);

        #[derive(Clone, Copy, Debug, PartialEq, PeekPoke)]
        struct SpatialId(pub usize, pub PipelineId);

        the_same(PipelineId(42, 2));
        the_same(ClipChainId(19u64, PipelineId(42, 2)));
        the_same(SpatialId(19usize, PipelineId(42, 2)));
    }
}