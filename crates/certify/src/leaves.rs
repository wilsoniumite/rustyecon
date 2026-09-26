//! Every scalar a value serialises, with its path: the machinery under the finite scan
//! (docs/CERTIFY.md §8, N12) and the determinism battery's comparison of two reports.
//!
//! It is a serde `Serializer` that writes nothing but a list. A struct field extends the path by
//! `.field` (just `field` at the root), a sequence element by `[i]`, a tuple element and a tuple
//! variant's field by `.i`, and an enum variant that carries data by `.Variant`. `Some(x)` is
//! transparent and `None` is a leaf. So the path of a number is where it sits in the RON a
//! certificate is written as. July's scan read chosen state fields, not the numbers its
//! certificate rendered (`v2p3: certify/nan.rs:47-57`); this one reads every number there is.

use serde::ser::{self, Serialize};
use std::fmt;

/// One scalar a value serialised.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Leaf {
    /// A float, as it was (an `f32` widened).
    F64(f64),
    /// Anything else, as text: a bool, an integer, a string, a unit variant's name, `None`.
    Other(String),
}

impl Leaf {
    /// Whether two leaves are the same, a float by its bits.
    pub(crate) fn same(&self, other: &Leaf) -> bool {
        match (self, other) {
            (Leaf::F64(a), Leaf::F64(b)) => a.to_bits() == b.to_bits(),
            (Leaf::Other(a), Leaf::Other(b)) => a == b,
            _ => false,
        }
    }
}

/// Every leaf of `v` with its path, in serialisation order. A value whose `Serialize` fails
/// yields the leaves before the failure and one leaf naming it, `Other("serialise: ..")` at the
/// root, so the failure is never silent.
pub(crate) fn leaves<T: Serialize + ?Sized>(v: &T) -> Vec<(String, Leaf)> {
    let mut out = Vec::new();
    if let Err(e) = v.serialize(Flat {
        out: &mut out,
        path: String::new(),
    }) {
        out.push((String::new(), Leaf::Other(format!("serialise: {e}"))));
    }
    out
}

/// A serialisation that cannot be flattened.
#[derive(Debug)]
pub(crate) struct FlatError(String);

impl fmt::Display for FlatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for FlatError {}

impl ser::Error for FlatError {
    fn custom<M: fmt::Display>(msg: M) -> FlatError {
        FlatError(msg.to_string())
    }
}

fn field(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{path}.{name}")
    }
}

struct Flat<'a> {
    out: &'a mut Vec<(String, Leaf)>,
    path: String,
}

impl Flat<'_> {
    fn leaf(self, l: Leaf) -> Result<(), FlatError> {
        self.out.push((self.path, l));
        Ok(())
    }

    fn other(self, s: impl fmt::Display) -> Result<(), FlatError> {
        self.leaf(Leaf::Other(s.to_string()))
    }
}

/// A compound value: its elements or fields, each under the compound's path.
struct Many<'a> {
    out: &'a mut Vec<(String, Leaf)>,
    path: String,
    i: usize,
    /// A map's current key, as text.
    key: String,
}

impl Many<'_> {
    fn next<T: Serialize + ?Sized>(&mut self, v: &T, path: String) -> Result<(), FlatError> {
        self.i += 1;
        v.serialize(Flat {
            out: self.out,
            path,
        })
    }

    fn indexed<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        let path = format!("{}[{}]", self.path, self.i);
        self.next(v, path)
    }

    fn dotted<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        let path = field(&self.path, &self.i.to_string());
        self.next(v, path)
    }
}

impl<'a> ser::Serializer for Flat<'a> {
    type Ok = ();
    type Error = FlatError;
    type SerializeSeq = Many<'a>;
    type SerializeTuple = Many<'a>;
    type SerializeTupleStruct = Many<'a>;
    type SerializeTupleVariant = Many<'a>;
    type SerializeMap = Many<'a>;
    type SerializeStruct = Many<'a>;
    type SerializeStructVariant = Many<'a>;

    fn serialize_bool(self, v: bool) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_i8(self, v: i8) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_i16(self, v: i16) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_i32(self, v: i32) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_i64(self, v: i64) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_i128(self, v: i128) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_u8(self, v: u8) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_u16(self, v: u16) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_u32(self, v: u32) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_u64(self, v: u64) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_u128(self, v: u128) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_f32(self, v: f32) -> Result<(), FlatError> {
        self.leaf(Leaf::F64(f64::from(v)))
    }
    fn serialize_f64(self, v: f64) -> Result<(), FlatError> {
        self.leaf(Leaf::F64(v))
    }
    fn serialize_char(self, v: char) -> Result<(), FlatError> {
        self.other(v)
    }
    fn serialize_str(self, v: &str) -> Result<(), FlatError> {
        self.other(format!("{v:?}"))
    }
    fn serialize_bytes(self, v: &[u8]) -> Result<(), FlatError> {
        self.other(format!("{v:?}"))
    }
    fn serialize_none(self) -> Result<(), FlatError> {
        self.other("None")
    }
    fn serialize_some<T: Serialize + ?Sized>(self, v: &T) -> Result<(), FlatError> {
        v.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), FlatError> {
        self.other("()")
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), FlatError> {
        self.other(name)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), FlatError> {
        self.other(variant)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        v: &T,
    ) -> Result<(), FlatError> {
        v.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        v: &T,
    ) -> Result<(), FlatError> {
        let path = field(&self.path, variant);
        v.serialize(Flat {
            out: self.out,
            path,
        })
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Many<'a>, FlatError> {
        Ok(Many {
            out: self.out,
            path: self.path,
            i: 0,
            key: String::new(),
        })
    }
    fn serialize_tuple(self, _: usize) -> Result<Many<'a>, FlatError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_struct(self, _: &'static str, _: usize) -> Result<Many<'a>, FlatError> {
        self.serialize_seq(None)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Many<'a>, FlatError> {
        Ok(Many {
            path: field(&self.path, variant),
            out: self.out,
            i: 0,
            key: String::new(),
        })
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Many<'a>, FlatError> {
        self.serialize_seq(None)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Many<'a>, FlatError> {
        self.serialize_seq(None)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Many<'a>, FlatError> {
        self.serialize_tuple_variant("", 0, variant, 0)
    }
}

impl ser::SerializeSeq for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        self.indexed(v)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeTuple for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        self.dotted(v)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeTupleStruct for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        self.dotted(v)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeTupleVariant for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_field<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        self.dotted(v)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeMap for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_key<T: Serialize + ?Sized>(&mut self, key: &T) -> Result<(), FlatError> {
        // A key is named by its own leaves, joined; the value follows under it.
        let named: Vec<String> = leaves(key)
            .into_iter()
            .map(|(_, l)| match l {
                Leaf::F64(x) => format!("{x:?}"),
                Leaf::Other(s) => s,
            })
            .collect();
        self.key = named.join(",");
        Ok(())
    }
    fn serialize_value<T: Serialize + ?Sized>(&mut self, v: &T) -> Result<(), FlatError> {
        let path = format!("{}[{}]", self.path, self.key);
        self.next(v, path)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeStruct for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), FlatError> {
        let path = field(&self.path, key);
        self.next(v, path)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

impl ser::SerializeStructVariant for Many<'_> {
    type Ok = ();
    type Error = FlatError;
    fn serialize_field<T: Serialize + ?Sized>(
        &mut self,
        key: &'static str,
        v: &T,
    ) -> Result<(), FlatError> {
        let path = field(&self.path, key);
        self.next(v, path)
    }
    fn end(self) -> Result<(), FlatError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    enum E {
        Unit,
        New(f64),
        Tup(u8, f64),
        Rec { x: f64 },
    }

    #[derive(Serialize)]
    struct S {
        a: f64,
        b: Vec<(u64, f64)>,
        c: Option<f64>,
        d: Option<f64>,
        e: Vec<E>,
        f: String,
    }

    #[test]
    fn every_scalar_has_its_path() {
        let s = S {
            a: 1.5,
            b: vec![(7, 2.5)],
            c: Some(3.5),
            d: None,
            e: vec![E::Unit, E::New(4.5), E::Tup(9, 5.5), E::Rec { x: 6.5 }],
            f: "x".into(),
        };
        let got: Vec<(String, Leaf)> = leaves(&s);
        let want = vec![
            ("a", Leaf::F64(1.5)),
            ("b[0].0", Leaf::Other("7".into())),
            ("b[0].1", Leaf::F64(2.5)),
            ("c", Leaf::F64(3.5)),
            ("d", Leaf::Other("None".into())),
            ("e[0]", Leaf::Other("Unit".into())),
            ("e[1].New", Leaf::F64(4.5)),
            ("e[2].Tup.0", Leaf::Other("9".into())),
            ("e[2].Tup.1", Leaf::F64(5.5)),
            ("e[3].Rec.x", Leaf::F64(6.5)),
            ("f", Leaf::Other("\"x\"".into())),
        ];
        let want: Vec<(String, Leaf)> = want.into_iter().map(|(p, l)| (p.into(), l)).collect();
        assert_eq!(got, want);
        let mut m = std::collections::BTreeMap::new();
        m.insert("k", 0.25);
        m.insert("l", 0.75);
        assert_eq!(
            leaves(&m),
            vec![
                ("[\"k\"]".to_string(), Leaf::F64(0.25)),
                ("[\"l\"]".to_string(), Leaf::F64(0.75))
            ]
        );
    }
}
